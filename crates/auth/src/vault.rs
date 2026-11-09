use crate::AuthResult;
use crate::suite::{self, Suite};
use crate::wire::{Reader, Writer};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use opaque_ke::ServerSetup;
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroizing;

mod layout
{
    pub const MAGIC: &[u8; 8] = b"B173VLT1";
    pub const SALT: usize = 16;
    pub const NONCE: usize = 24;
    pub const KEY: usize = 32;
    pub const HEADER: usize = MAGIC.len() + SALT;
}

mod invites
{
    // no 0 O 1 I so a code read aloud cannot be mistyped, 32 symbols keep the byte modulo unbiased
    pub const ALPHABET: &[u8; 32] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
    pub const LEN: usize = 10;
    pub const LIFETIME_SECS: u64 = 24 * 60 * 60;
}

const PASSPHRASE_MIN: usize = 12;

pub struct Account
{
    pub m_display: String,
    pub m_suite: u8,
    pub m_record: Vec<u8>,
    pub m_created: u64,
}

struct Invite
{
    m_name: String,
    m_digest: [u8; 32],
    m_expires: u64,
}

// server key, accounts and invites, sealed as one file under a key stretched from the operator passphrase
pub struct Vault
{
    m_path: PathBuf,
    m_salt: [u8; layout::SALT],
    m_key: Zeroizing<[u8; layout::KEY]>,
    m_setup: ServerSetup<Suite>,
    m_accounts: BTreeMap<String, Account>,
    m_invites: Vec<Invite>,
}

impl Vault
{
    pub fn open(path: &Path, passphrase: &[u8]) -> AuthResult<Vault>
    {
        if passphrase.len() < PASSPHRASE_MIN
        {
            return Err(format!("the auth passphrase needs at least {PASSPHRASE_MIN} characters"));
        }

        match fs::read(path)
        {
            Ok(data) => Self::decrypt(path, &data, passphrase),
            Err(error) if error.kind() == ErrorKind::NotFound => Self::create(path, passphrase),
            Err(error) => Err(format!("cannot read auth vault {}: {error}", path.display())),
        }
    }

    fn create(path: &Path, passphrase: &[u8]) -> AuthResult<Vault>
    {
        let mut salt: [u8; layout::SALT] = [0; layout::SALT];
        OsRng.fill_bytes(&mut salt);
        let vault: Vault = Vault {
            m_path: path.to_path_buf(),
            m_salt: salt,
            m_key: stretch(passphrase, &salt)?,
            m_setup: ServerSetup::<Suite>::new(&mut OsRng),
            m_accounts: BTreeMap::new(),
            m_invites: Vec::new(),
        };
        vault.save()?;
        Ok(vault)
    }

    fn decrypt(path: &Path, data: &[u8], passphrase: &[u8]) -> AuthResult<Vault>
    {
        let damaged: String = format!("auth vault {} is damaged", path.display());
        if data.len() < layout::HEADER + layout::NONCE || &data[..layout::MAGIC.len()] != layout::MAGIC
        {
            return Err(damaged);
        }

        let salt: [u8; layout::SALT] = data[layout::MAGIC.len()..layout::HEADER].try_into().map_err(|_| &damaged)?;
        let key: Zeroizing<[u8; layout::KEY]> = stretch(passphrase, &salt)?;
        let nonce: &XNonce = XNonce::from_slice(&data[layout::HEADER..layout::HEADER + layout::NONCE]);
        let payload: Payload<'_, '_> =
            Payload { msg: &data[layout::HEADER + layout::NONCE..], aad: &data[..layout::HEADER] };
        let plain: Zeroizing<Vec<u8>> = Zeroizing::new(cipher(&key).decrypt(nonce, payload).map_err(|_| {
            format!("auth vault {} could not be decrypted, wrong passphrase or damaged file", path.display())
        })?);

        let mut reader: Reader<'_> = Reader::new(&plain);
        let setup: ServerSetup<Suite> = ServerSetup::<Suite>::deserialize(&Zeroizing::new(reader.bytes()?))
            .map_err(|_| format!("auth vault {} holds a bad server key", path.display()))?;

        let mut accounts: BTreeMap<String, Account> = BTreeMap::new();
        for _ in 0..reader.long()?
        {
            let name: String = reader.text()?;
            let account: Account = Account {
                m_display: reader.text()?,
                m_suite: reader.byte()?,
                m_record: reader.bytes()?,
                m_created: reader.long()?,
            };
            accounts.insert(name, account);
        }

        let mut invites: Vec<Invite> = Vec::new();
        for _ in 0..reader.long()?
        {
            let name: String = reader.text()?;
            let digest: [u8; 32] = reader.bytes()?.try_into().map_err(|_| &damaged)?;
            invites.push(Invite { m_name: name, m_digest: digest, m_expires: reader.long()? });
        }
        reader.end()?;

        Ok(Vault {
            m_path: path.to_path_buf(),
            m_salt: salt,
            m_key: key,
            m_setup: setup,
            m_accounts: accounts,
            m_invites: invites,
        })
    }

    fn serialize(&self) -> Zeroizing<Vec<u8>>
    {
        let mut writer: Writer = Writer::new();
        writer.bytes(&Zeroizing::new(self.m_setup.serialize().to_vec()));
        writer.long(self.m_accounts.len() as u64);
        for (name, account) in &self.m_accounts
        {
            writer.text(name);
            writer.text(&account.m_display);
            writer.byte(account.m_suite);
            writer.bytes(&account.m_record);
            writer.long(account.m_created);
        }
        writer.long(self.m_invites.len() as u64);
        for invite in &self.m_invites
        {
            writer.text(&invite.m_name);
            writer.bytes(&invite.m_digest);
            writer.long(invite.m_expires);
        }
        Zeroizing::new(writer.finish())
    }

    fn save(&self) -> AuthResult<()>
    {
        let plain: Zeroizing<Vec<u8>> = self.serialize();
        let mut nonce: [u8; layout::NONCE] = [0; layout::NONCE];
        OsRng.fill_bytes(&mut nonce);

        let mut file: Vec<u8> = Vec::with_capacity(layout::HEADER + layout::NONCE + plain.len());
        file.extend_from_slice(layout::MAGIC);
        file.extend_from_slice(&self.m_salt);
        let payload: Payload<'_, '_> = Payload { msg: &plain, aad: &file };
        let sealed: Vec<u8> = cipher(&self.m_key)
            .encrypt(XNonce::from_slice(&nonce), payload)
            .map_err(|_| String::from("auth vault failed to encrypt"))?;
        file.extend_from_slice(&nonce);
        file.extend_from_slice(&sealed);
        writeAtomic(&self.m_path, &file)
    }

    pub fn setup(&self) -> &ServerSetup<Suite> { &self.m_setup }

    pub fn key(&self) -> Vec<u8> { self.m_setup.keypair().public().serialize().to_vec() }

    pub fn account(&self, name: &str) -> Option<&Account> { self.m_accounts.get(name) }

    pub fn count(&self) -> usize { self.m_accounts.len() }

    // writes through to disk and keeps memory unchanged when the write fails
    pub fn store(&mut self, name: &str, account: Account) -> AuthResult<()>
    {
        let previous: Option<Account> = self.m_accounts.insert(name.to_string(), account);
        let saved: AuthResult<()> = self.save();
        if saved.is_err()
        {
            match previous
            {
                Some(previous) => self.m_accounts.insert(name.to_string(), previous),
                None => self.m_accounts.remove(name),
            };
        }
        saved
    }

    pub fn remove(&mut self, name: &str) -> AuthResult<bool>
    {
        let Some(previous) = self.m_accounts.remove(name)
        else
        {
            return Ok(false);
        };

        let saved: AuthResult<()> = self.save();
        if saved.is_err()
        {
            self.m_accounts.insert(name.to_string(), previous);
        }
        saved.map(|()| true)
    }

    pub fn invite(&mut self, name: &str) -> AuthResult<String>
    {
        let mut bytes: Zeroizing<[u8; invites::LEN]> = Zeroizing::new([0; invites::LEN]);
        OsRng.fill_bytes(bytes.as_mut());
        let code: String =
            bytes.iter().map(|byte| invites::ALPHABET[*byte as usize % invites::ALPHABET.len()] as char).collect();

        let now: u64 = unixNow();
        self.m_invites.retain(|invite| invite.m_expires > now && invite.m_name != name);
        self.m_invites.push(Invite {
            m_name: name.to_string(),
            m_digest: digest(name, &code),
            m_expires: now + invites::LIFETIME_SECS,
        });
        self.save()?;
        Ok(code)
    }

    pub fn inviteValid(&self, name: &str, code: &str) -> bool { self.findInvite(name, code).is_some() }

    fn findInvite(&self, name: &str, code: &str) -> Option<usize>
    {
        let wanted: [u8; 32] = digest(name, &code.trim().to_ascii_uppercase());
        let now: u64 = unixNow();
        self.m_invites
            .iter()
            .position(|invite| invite.m_name == name && invite.m_expires > now && invite.m_digest == wanted)
    }

    pub fn redeem(&mut self, name: &str, code: &str) -> AuthResult<bool>
    {
        let Some(index) = self.findInvite(name, code)
        else
        {
            return Ok(false);
        };

        self.m_invites.remove(index);
        self.save().map(|()| true)
    }
}

fn cipher(key: &[u8; layout::KEY]) -> XChaCha20Poly1305 { XChaCha20Poly1305::new(Key::from_slice(key)) }

fn stretch(passphrase: &[u8], salt: &[u8]) -> AuthResult<Zeroizing<[u8; layout::KEY]>>
{
    let mut key: Zeroizing<[u8; layout::KEY]> = Zeroizing::new([0; layout::KEY]);
    suite::argon2()
        .hash_password_into(passphrase, salt, key.as_mut())
        .map_err(|error| format!("cannot derive the auth vault key: {error}"))?;
    Ok(key)
}

fn digest(name: &str, code: &str) -> [u8; 32]
{
    let mut hasher: Sha256 = Sha256::new();
    hasher.update(name.as_bytes());
    hasher.update(b":");
    hasher.update(code.as_bytes());
    hasher.finalize().into()
}

pub fn unixNow() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs()) }

fn ioError(path: &Path, action: &str) -> impl FnOnce(std::io::Error) -> String
{
    let path: String = path.display().to_string();
    let action: String = action.to_string();
    move |error| format!("cannot {action} {path}: {error}")
}

// temp file then rename so a crash mid write never leaves a half vault
fn writeAtomic(path: &Path, data: &[u8]) -> AuthResult<()>
{
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty())
    {
        createDir(parent)?;
    }

    let temp: PathBuf = path.with_extension("tmp");
    match fs::remove_file(&temp)
    {
        Err(error) if error.kind() != ErrorKind::NotFound => return Err(ioError(&temp, "remove")(error)),
        _ =>
        {}
    }

    let mut options: OpenOptions = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file: File = options.open(&temp).map_err(ioError(&temp, "create"))?;
    file.write_all(data).map_err(ioError(&temp, "write"))?;
    file.sync_all().map_err(ioError(&temp, "sync"))?;
    fs::rename(&temp, path).map_err(ioError(path, "replace"))
}

fn createDir(dir: &Path) -> AuthResult<()>
{
    let mut builder: fs::DirBuilder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(dir).map_err(ioError(dir, "create"))
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn tempPath(tag: &str) -> PathBuf
    {
        let mut nonce: [u8; 8] = [0; 8];
        OsRng.fill_bytes(&mut nonce);
        std::env::temp_dir().join(format!("b173-auth-{tag}-{}", u64::from_le_bytes(nonce))).join("vault.bin")
    }

    fn account(display: &str) -> Account
    {
        Account { m_display: display.to_string(), m_suite: suite::SUITE_VERSION, m_record: vec![1, 2, 3], m_created: 5 }
    }

    #[test]
    fn vault_survives_reopen_and_rejects_wrong_passphrase()
    {
        let path: PathBuf = tempPath("reopen");
        let mut vault: Vault = Vault::open(&path, b"correct horse battery").expect("create");
        vault.store("notch", account("Notch")).expect("store");
        let key: Vec<u8> = vault.key();

        let reopened: Vault = Vault::open(&path, b"correct horse battery").expect("reopen");
        assert_eq!(reopened.key(), key);
        assert_eq!(reopened.account("notch").map(|account| account.m_display.as_str()), Some("Notch"));
        assert!(Vault::open(&path, b"wrong horse battery").is_err());
        assert!(Vault::open(&path, b"short").is_err());

        let disk: Vec<u8> = fs::read(&path).expect("read");
        assert!(!disk.windows(5).any(|window| window == b"Notch"));
        #[cfg(unix)]
        assert_eq!(
            std::os::unix::fs::PermissionsExt::mode(&fs::metadata(&path).expect("stat").permissions()) & 0o777,
            0o600
        );
    }

    #[test]
    fn tampered_vault_does_not_open()
    {
        let path: PathBuf = tempPath("tamper");
        Vault::open(&path, b"correct horse battery").expect("create");
        let mut disk: Vec<u8> = fs::read(&path).expect("read");
        let last: usize = disk.len() - 1;
        disk[last] ^= 1;
        fs::write(&path, &disk).expect("write");
        assert!(Vault::open(&path, b"correct horse battery").is_err());
    }

    #[test]
    fn invites_are_bound_to_a_name_and_single_use()
    {
        let path: PathBuf = tempPath("invite");
        let mut vault: Vault = Vault::open(&path, b"correct horse battery").expect("create");
        let code: String = vault.invite("notch").expect("invite");
        assert!(!vault.inviteValid("jeb_", &code));
        assert!(vault.inviteValid("notch", &code.to_ascii_lowercase()));
        assert_eq!(vault.redeem("notch", &code), Ok(true));
        assert_eq!(vault.redeem("notch", &code), Ok(false));
    }
}
