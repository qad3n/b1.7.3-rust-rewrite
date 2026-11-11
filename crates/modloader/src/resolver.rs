use crate::manifest::{Manifest, SideSpec};
use mod_protocol::identifier::isValidModId;
use mod_protocol::{API_MAJOR, API_MINOR};
use semver::{Version, VersionReq};
use std::collections::{BTreeMap, BTreeSet};

pub const LOADER_DEPENDENCY: &str = "loader";
const RESERVED_IDS: [&str; 3] = ["minecraft", "loader", "c"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem
{
    pub m_mod: String,
    pub m_file: String,
    pub m_reason: String,
}

pub struct Candidate
{
    pub m_manifest: Manifest,
    pub m_file: String,
}

pub struct Resolution
{
    // indices into the candidate list in load order
    pub m_order: Vec<usize>,
    pub m_skipped: Vec<(String, String)>,
    pub m_problems: Vec<Problem>,
}

pub fn parseApi(api: &str) -> Option<(u32, u32)>
{
    let (major, minor) = api.split_once('.').unwrap_or((api, "0"));
    Some((major.parse().ok()?, minor.trim_end_matches(".x").parse().ok()?))
}

fn versionReq(text: &str) -> Result<VersionReq, String>
{
    let normalized: &str = if text.trim() == "*" { "*" } else { text };
    VersionReq::parse(normalized).map_err(|error| format!("bad version range '{text}': {error}"))
}

struct Resolver<'a>
{
    m_candidates: &'a [Candidate],
    m_physical: SideSpec,
    m_problems: Vec<Problem>,
}

impl Resolver<'_>
{
    fn problem(&mut self, index: usize, reason: String)
    {
        let candidate: &Candidate = &self.m_candidates[index];

        self.m_problems.push(Problem
        {
            m_mod: candidate.m_manifest.id.clone(),
            m_file: candidate.m_file.clone(),
            m_reason: reason,
        });
    }

    fn validate(&mut self, index: usize) -> bool
    {
        let manifest: &Manifest = &self.m_candidates[index].m_manifest;
        let mut reasons: Vec<String> = Vec::new();

        if !isValidModId(&manifest.id)
        {
            reasons.push(format!("mod id '{}' must match [a-z0-9_]{{2,64}}", manifest.id));
        }

        if RESERVED_IDS.contains(&manifest.id.as_str())
        {
            reasons.push(format!("mod id '{}' is reserved", manifest.id));
        }

        if let Err(error) = Version::parse(&manifest.version)
        {
            reasons.push(format!("version '{}' is not semver: {error}", manifest.version));
        }

        match parseApi(&manifest.api)
        {
            Some((major, minor)) if major == API_MAJOR && API_MINOR.checked_sub(minor).is_some() =>
            {}
            Some(_) =>
            {
                reasons.push(format!("needs loader API {} but this loader has {API_MAJOR}.{API_MINOR}", manifest.api))
            }

            None => reasons.push(format!("api '{}' is not major.minor", manifest.api)),
        }

        let failed: bool = !reasons.is_empty();

        for reason in reasons
        {
            self.problem(index, reason);
        }

        !failed
    }

    fn matchesSide(&self, index: usize) -> bool
    {
        let side: SideSpec = self.m_candidates[index].m_manifest.side;
        side == SideSpec::Both || side == self.m_physical
    }
}

// one id may come from two files only when a mod owns it and another provides it, then the higher version wins
// two providers with no owner are rival implementations, both are refused so the player picks one
fn pickDuplicates(resolver: &mut Resolver<'_>, active: &mut BTreeSet<usize>)
{
    let mut owners: BTreeMap<String, Vec<usize>> = BTreeMap::new();

    for index in active.iter().copied()
    {
        let manifest: &Manifest = &resolver.m_candidates[index].m_manifest;
        owners.entry(manifest.id.clone()).or_default().push(index);

        for alias in &manifest.provides
        {
            owners.entry(alias.clone()).or_default().push(index);
        }
    }

    for (id, indices) in owners
    {
        // an earlier group may already have refused some of these files
        let indices: Vec<usize> = indices.into_iter().filter(|index| active.contains(index)).collect();
        if indices.len() < 2
        {
            continue;
        }

        let owned: bool = indices.iter().any(|index| resolver.m_candidates[*index].m_manifest.id == id);
        let provided: bool =
            indices.iter().any(|index| resolver.m_candidates[*index].m_manifest.provides.contains(&id));
        let files: Vec<String> = indices.iter().map(|index| resolver.m_candidates[*index].m_file.clone()).collect();

        if owned && provided
        {
            let best: usize = *indices
                .iter()
                .max_by_key(|index| Version::parse(&resolver.m_candidates[**index].m_manifest.version).ok())
                .expect("a duplicate set has members");

            for index in indices.iter().copied().filter(|index| *index != best)
            {
                resolver.problem(
                    index,
                    format!("'{id}' also comes from {}, which wins by version", resolver.m_candidates[best].m_file),
                );
                active.remove(&index);
            }

            continue;
        }

        let reason: String = match owned
        {
            true => format!("mod id '{id}' is used by several files: {}", files.join(", ")),
            false => format!("'{id}' is provided by several mods, keep one of: {}", files.join(", ")),
        };

        for index in indices
        {
            resolver.problem(index, reason.clone());
            active.remove(&index);
        }
    }
}

fn findProvider(candidates: &[Candidate], active: &BTreeSet<usize>, id: &str) -> Option<usize>
{
    active.iter().copied().find(|index|
    {
        let manifest: &Manifest = &candidates[*index].m_manifest;
        manifest.id == id || manifest.provides.iter().any(|alias| alias == id)
    })
}

fn checkRelations(resolver: &mut Resolver<'_>, active: &BTreeSet<usize>) -> BTreeSet<usize>
{
    let loader: Version = Version::parse(crate::LOADER_VERSION).expect("the crate version is semver");
    let mut failed: BTreeSet<usize> = BTreeSet::new();

    for index in active.iter().copied()
    {
        let manifest: Manifest = resolver.m_candidates[index].m_manifest.clone();
        for (dependency, range) in &manifest.depends
        {
            let requirement: VersionReq = match versionReq(range)
            {
                Ok(requirement) => requirement,

                Err(reason) =>
                {
                    resolver.problem(index, reason);
                    failed.insert(index);
                    continue;
                }
            };

            if dependency == LOADER_DEPENDENCY
            {
                if !requirement.matches(&loader)
                {
                    resolver.problem(index, format!("needs loader {range}, this is {loader}"));
                    failed.insert(index);
                }

                continue;
            }

            match findProvider(resolver.m_candidates, active, dependency)
            {
                Some(provider) =>
                {
                    let version: Option<Version> =
                        Version::parse(&resolver.m_candidates[provider].m_manifest.version).ok();

                    if !version.as_ref().is_some_and(|version| requirement.matches(version))
                    {
                        let found: String = resolver.m_candidates[provider].m_manifest.version.clone();
                        resolver.problem(index, format!("needs {dependency} {range}, found {found}"));
                        failed.insert(index);
                    }
                }

                None =>
                {
                    resolver.problem(index, format!("needs {dependency} {range}, which is not installed"));
                    failed.insert(index);
                }
            }
        }

        for (broken, range) in &manifest.breaks
        {
            let requirement: VersionReq = match versionReq(range)
            {
                Ok(requirement) => requirement,

                Err(reason) =>
                {
                    resolver.problem(index, reason);
                    failed.insert(index);
                    continue;
                }
            };

            if let Some(other) = findProvider(resolver.m_candidates, active, broken)
                && Version::parse(&resolver.m_candidates[other].m_manifest.version)
                    .is_ok_and(|version| requirement.matches(&version))
            {
                resolver.problem(index, format!("is incompatible with {broken} {range}, remove one of them"));
                failed.insert(index);
            }
        }
    }

    failed
}

// a mod whose dependency failed fails too, so the report names the root cause and every casualty
fn cascade(resolver: &mut Resolver<'_>, active: &mut BTreeSet<usize>, mut failed: BTreeSet<usize>)
{
    while !failed.is_empty()
    {
        for index in &failed
        {
            active.remove(index);
        }

        let mut next: BTreeSet<usize> = BTreeSet::new();

        for index in active.iter().copied()
        {
            let manifest: Manifest = resolver.m_candidates[index].m_manifest.clone();
            for dependency in manifest.depends.keys().filter(|dependency| *dependency != LOADER_DEPENDENCY)
            {
                if findProvider(resolver.m_candidates, active, dependency).is_none()
                    && failed.iter().any(|gone|
                    {
                        let goneManifest: &Manifest = &resolver.m_candidates[*gone].m_manifest;
                        goneManifest.id == *dependency || goneManifest.provides.contains(dependency)
                    })
                {
                    resolver.problem(index, format!("needs {dependency}, which failed to load"));
                    next.insert(index);
                }
            }
        }

        failed = next;
    }
}

fn sort(resolver: &mut Resolver<'_>, active: &BTreeSet<usize>) -> Vec<usize>
{
    let mut edges: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
    let mut incoming: BTreeMap<usize, usize> = active.iter().map(|index| (*index, 0)).collect();

    let mut addEdge = |from: usize, to: usize, edges: &mut BTreeMap<usize, BTreeSet<usize>>|
    {
        if from != to && edges.entry(from).or_default().insert(to)
        {
            *incoming.get_mut(&to).expect("both ends are active") += 1;
        }
    };

    for index in active.iter().copied()
    {
        let manifest: &Manifest = &resolver.m_candidates[index].m_manifest;
        let befores: Vec<&String> =
            manifest.depends.keys().chain(manifest.optional.keys()).chain(manifest.load.after.iter()).collect();

        for other in befores
        {
            if let Some(provider) = findProvider(resolver.m_candidates, active, other)
            {
                addEdge(provider, index, &mut edges);
            }
        }

        for other in &manifest.load.before
        {
            if let Some(provider) = findProvider(resolver.m_candidates, active, other)
            {
                addEdge(index, provider, &mut edges);
            }
        }
    }

    let idOf = |index: usize| resolver.m_candidates[index].m_manifest.id.clone();
    let mut ready: BTreeSet<(String, usize)> =
        incoming.iter().filter(|(_, count)| **count == 0).map(|(index, _)| (idOf(*index), *index)).collect();
    let mut order: Vec<usize> = Vec::new();

    while let Some(first) = ready.pop_first()
    {
        order.push(first.1);

        for next in edges.get(&first.1).cloned().unwrap_or_default()
        {
            let count: &mut usize = incoming.get_mut(&next).expect("edge targets are active");
            *count -= 1;
            if *count == 0
            {
                ready.insert((idOf(next), next));
            }
        }
    }

    let cycle: Vec<usize> = active.iter().copied().filter(|index| !order.contains(index)).collect();
    let names: Vec<String> = cycle.iter().map(|index| idOf(*index)).collect();

    for index in cycle
    {
        resolver.problem(index, format!("load order cycle between {}", names.join(", ")));
    }

    order
}

pub fn resolve(candidates: &[Candidate], physical: SideSpec) -> Resolution
{
    let mut resolver: Resolver<'_> =
        Resolver { m_candidates: candidates, m_physical: physical, m_problems: Vec::new() };
    let mut skipped: Vec<(String, String)> = Vec::new();
    let mut active: BTreeSet<usize> = BTreeSet::new();

    for index in 0..candidates.len()
    {
        if !resolver.validate(index)
        {
            continue;
        }

        if !resolver.matchesSide(index)
        {
            skipped.push((
                candidates[index].m_manifest.id.clone(),
                format!("{:?} only", candidates[index].m_manifest.side),
            ));
            continue;
        }

        active.insert(index);
    }

    pickDuplicates(&mut resolver, &mut active);

    let failed: BTreeSet<usize> = checkRelations(&mut resolver, &active);
    cascade(&mut resolver, &mut active, failed);

    let order: Vec<usize> = sort(&mut resolver, &active);
    Resolution { m_order: order, m_skipped: skipped, m_problems: resolver.m_problems }
}
