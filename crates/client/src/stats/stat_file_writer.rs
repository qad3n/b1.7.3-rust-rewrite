use crate::argo::{JdomParser, JsonFields, JsonNode, JsonRootNode, JsonStringNode, PathElement};
use crate::client::Session;
use crate::stats::MD5String;
use crate::stats::stats_syncher::StatsSyncher;
use game::stats::{Achievement, StatBase, StatList};
use jvm::field::Field;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub type StatMap = Vec<(Rc<StatBase>, i32)>;

pub fn get(values: &StatMap, stat: &Rc<StatBase>) -> Option<i32>
{
    values.iter().find(|(key, _)| Rc::ptr_eq(key, stat)).map(|(_, value)| *value)
}

pub fn put(values: &mut StatMap, stat: &Rc<StatBase>, value: i32)
{
    match values.iter_mut().find(|(key, _)| Rc::ptr_eq(key, stat))
    {
        Some(entry) => entry.1 = value,
        None => values.push((Rc::clone(stat), value)),
    }
}

pub struct StatFileWriter
{
    m_stats: RefCell<StatMap>,
    m_unsentStats: RefCell<StatMap>,
    m_dirty: Cell<bool>,
    m_statsSyncher: Field<Option<Rc<StatsSyncher>>>,
}

impl StatFileWriter
{
    pub fn new(session: &Rc<Session>, parentDir: &Path) -> StatFileWriter
    {
        let file1: PathBuf = parentDir.join("stats");
        if !file1.exists()
        {
            let _created: bool = std::fs::create_dir(&file1).is_ok();
        }
        let listed: Vec<PathBuf> = std::fs::read_dir(parentDir)
            .ok()
            .orNpe()
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .collect();
        for file2 in listed
        {
            let name: String = file2.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
            if name.starts_with("stats_") && name.ends_with(".dat")
            {
                let file3: PathBuf = file1.join(&name);
                if !file3.exists()
                {
                    jvm::system::println(&format!("Relocating {name}"));
                    let _renamed: bool = std::fs::rename(&file2, &file3).is_ok();
                }
            }
        }
        let writer: StatFileWriter = StatFileWriter {
            m_stats: RefCell::new(Vec::new()),
            m_unsentStats: RefCell::new(Vec::new()),
            m_dirty: Cell::new(false),
            m_statsSyncher: Field::new(None),
        };
        let syncher: StatsSyncher = StatsSyncher::new(session, &writer, &file1);
        writer.m_statsSyncher.set(Some(Rc::new(syncher)));
        writer
    }

    fn statsSyncher(&self) -> Rc<StatsSyncher> { self.m_statsSyncher.get().orNpe() }

    pub fn readStat(&self, stat: &Rc<StatBase>, amount: i32)
    {
        Self::writeStatToMap(&mut self.m_unsentStats.borrow_mut(), stat, amount);
        Self::writeStatToMap(&mut self.m_stats.borrow_mut(), stat, amount);
        self.m_dirty.set(true);
    }

    fn writeStatToMap(values: &mut StatMap, stat: &Rc<StatBase>, amount: i32)
    {
        let n2: i32 = get(values, stat).unwrap_or(0);
        put(values, stat, n2.wrapping_add(amount));
    }

    pub fn getUnsent(&self) -> StatMap { self.m_unsentStats.borrow().clone() }

    pub fn increment(&self, amounts: Option<StatMap>)
    {
        let Some(amounts) = amounts
        else
        {
            return;
        };
        self.m_dirty.set(true);
        for (stat, amount) in &amounts
        {
            Self::writeStatToMap(&mut self.m_unsentStats.borrow_mut(), stat, *amount);
            Self::writeStatToMap(&mut self.m_stats.borrow_mut(), stat, *amount);
        }
    }

    pub fn tickStats(&self, stats: Option<StatMap>)
    {
        let Some(stats) = stats
        else
        {
            return;
        };
        for (stat, value) in &stats
        {
            let n2: i32 = get(&self.m_unsentStats.borrow(), stat).unwrap_or(0);
            put(&mut self.m_stats.borrow_mut(), stat, value.wrapping_add(n2));
        }
    }

    pub fn tickUnsent(&self, stats: Option<StatMap>)
    {
        let Some(stats) = stats
        else
        {
            return;
        };
        self.m_dirty.set(true);
        for (stat, value) in &stats
        {
            Self::writeStatToMap(&mut self.m_unsentStats.borrow_mut(), stat, *value);
        }
    }

    pub fn deserialize(data: &str) -> Option<StatMap>
    {
        let mut hashMap: StatMap = Vec::new();
        let string: &str = "local";
        let mut stringBuilder: String = String::new();
        let root: JsonRootNode = match JdomParser.parse(&JString::from(data))
        {
            Ok(root) => root,
            Err(exception) =>
            {
                exception.printStackTrace();
                return Some(hashMap);
            }
        };
        for object2 in root.getArrayNode(&[PathElement::Field("stats-change".to_string())])
        {
            let map: JsonFields = object2.getFields();
            let Some((key, value)): Option<(JsonStringNode, Rc<JsonNode>)> = map.entries().into_iter().next()
            else
            {
                jvm::throwable::throwBare(&jvm::throwable::NO_SUCH_ELEMENT)
            };
            let n: i32 =
                jvm::integer::parseInt(&key.getText()).unwrap_or_else(|exception| jvm::throwable::rethrow(exception));
            let n2: i32 =
                jvm::integer::parseInt(&value.getText()).unwrap_or_else(|exception| jvm::throwable::rethrow(exception));
            let Some(statBase) = StatList::byKey(n)
            else
            {
                jvm::system::println(&format!("{n} is not a valid stat"));
                continue;
            };
            let guid: Option<String> = StatList::byKey(n).orNpe().m_statGuid.get();
            stringBuilder.push_str(&format!("{},{n2},", guid.as_deref().unwrap_or("null")));
            put(&mut hashMap, &statBase, n2);
        }
        let mD5String: MD5String = MD5String::new(string);
        let string2: String = mD5String.digest(&stringBuilder);
        if JString::from(string2.as_str()) != root.getStringValue(&[PathElement::Field("checksum".to_string())])
        {
            jvm::system::println("CHECKSUM MISMATCH");
            return None;
        }
        Some(hashMap)
    }

    pub fn serialize(playerName: Option<&str>, sessionId: Option<&str>, stats: &StatMap) -> String
    {
        let mut stringBuilder: String = String::new();
        let mut stringBuilder2: String = String::new();
        let mut bl: bool = true;
        stringBuilder.push_str("{\r\n");
        if let (Some(playerName), Some(sessionId)) = (playerName, sessionId)
        {
            stringBuilder.push_str("  \"user\":{\r\n");
            stringBuilder.push_str(&format!("    \"name\":\"{playerName}\",\r\n"));
            stringBuilder.push_str(&format!("    \"sessionid\":\"{sessionId}\"\r\n"));
            stringBuilder.push_str("  },\r\n");
        }
        stringBuilder.push_str("  \"stats-change\":[");
        for (stat, value) in stats
        {
            if !bl
            {
                stringBuilder.push_str("},");
            }
            else
            {
                bl = false;
            }
            stringBuilder.push_str(&format!("\r\n    {{\"{}\":{value}", stat.m_statId));
            stringBuilder2.push_str(&format!("{},{value},", stat.m_statGuid.get().as_deref().unwrap_or("null")));
        }
        if !bl
        {
            stringBuilder.push('}');
        }
        let mD5String: MD5String = MD5String::new(sessionId.unwrap_or("null"));
        stringBuilder.push_str("\r\n  ],\r\n");
        stringBuilder.push_str(&format!("  \"checksum\":\"{}\"\r\n", mD5String.digest(&stringBuilder2)));
        stringBuilder.push('}');
        stringBuilder
    }

    pub fn hasAchievementUnlocked(&self, achievement: &Rc<StatBase>) -> bool
    {
        get(&self.m_stats.borrow(), achievement).is_some()
    }

    pub fn hasParentAchievement(&self, achievement: &Rc<StatBase>) -> bool
    {
        match &Achievement::of(achievement).m_parentAchievement
        {
            None => true,
            Some(parent) => self.hasAchievementUnlocked(parent),
        }
    }

    pub fn writeStat(&self, stat: &Rc<StatBase>) -> i32 { get(&self.m_stats.borrow(), stat).unwrap_or(0) }

    pub fn func_27175_b(&self) {}

    pub fn syncStats(&self) { self.statsSyncher().syncStatsFileWithMap(&self.getUnsent()); }

    pub fn tick(&self)
    {
        let syncher: Rc<StatsSyncher> = self.statsSyncher();
        if self.m_dirty.get() && syncher.canSave()
        {
            syncher.save(&self.getUnsent());
        }
        syncher.tick(self);
    }
}
