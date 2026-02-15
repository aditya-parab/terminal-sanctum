#![allow(dead_code)]
use chrono::{DateTime, Timelike, Utc};
use directories::ProjectDirs;
use rand::Rng;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;

// --- Balance Constants ---
pub const MAX_SATIETY: f32 = 100.0;
pub const MAX_FOCUS: f32 = 100.0;
pub const FLOW_STATE_THRESHOLD: f32 = 80.0;
pub const SPECIALIZATION_LEVEL: u32 = 10;

const SATIETY_DECAY_PER_HOUR: f32 = 1.0;
const FOCUS_DECAY_PER_30_MIN: f32 = 1.0;
const FOCUS_GAIN_INTERACT: f32 = 15.0;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EvolutionStage {
    Egg,
    Blob,
    Dog,
    Dragon,
    Robot,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaceType {
    Orc,
    Human,
    Undead,
    NightElf,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Specialization {
    Blademaster,
    FarSeer,
    Paladin,
    Archmage,
    DeathKnight,
    Lich,
    DemonHunter,
    Druid,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Emote {
    None,
    Cheer,
    Roar,
    Dance,
    Salute,
    Ponder,
    Flex,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub description: String,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contribution {
    pub project: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistentState {
    pub name: String,
    pub race_type: RaceType,
    pub gender: Gender,
    pub specialization: Option<Specialization>,
    pub xp: u32,
    pub rested_xp: u32,
    pub last_saved_at: DateTime<Utc>,
    pub birth_date: DateTime<Utc>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
pub struct Avatar {
    pub name: String,
    pub race_type: RaceType,
    pub gender: Gender,
    pub specialization: Option<Specialization>,
    pub xp: u32,
    pub rested_xp: u32,
    pub last_saved_at: DateTime<Utc>,
    pub birth_date: DateTime<Utc>,
    pub satiety: f32,
    pub focus: f32,
    pub tasks: Vec<Task>,
    pub logs: Vec<LogEntry>,
    pub contributions: Vec<Contribution>,
    pub last_commit: DateTime<Utc>,
    pub last_interaction: DateTime<Utc>,
    pub current_emote: Emote,
    pub emote_end_time: Option<DateTime<Utc>>,
    pub ultimate_active_until: Option<DateTime<Utc>>,
}

impl Avatar {
    /// SUMMON: Principal Hero Factory.
    ///
    /// # Example
    /// ```
    /// use sanctum_core::Avatar;
    /// let avatar = Avatar::summon();
    /// assert_eq!(avatar.xp, 0);
    /// assert_eq!(avatar.level(), 1);
    /// ```
    pub fn summon() -> Self {
        let mut rng = rand::thread_rng();
        let race_type = match rng.gen_range(0..4) {
            0 => RaceType::Orc,
            1 => RaceType::Human,
            2 => RaceType::Undead,
            _ => RaceType::NightElf,
        };
        let gender = if rng.gen_bool(0.5) {
            Gender::Male
        } else {
            Gender::Female
        };
        let name = get_legendary_name(race_type, gender, &mut rng);
        let mut avatar = Self {
            name,
            race_type,
            gender,
            specialization: None,
            xp: 0,
            rested_xp: 0,
            last_saved_at: Utc::now(),
            birth_date: Utc::now(),
            satiety: MAX_SATIETY,
            focus: 50.0,
            tasks: Vec::new(),
            logs: Vec::new(),
            contributions: Vec::new(),
            last_commit: Utc::now(),
            last_interaction: Utc::now(),
            current_emote: Emote::None,
            emote_end_time: None,
            ultimate_active_until: None,
        };
        avatar.add_log(get_intro_message(race_type, &avatar.name));
        avatar
    }

    /// REINCARNATION: Restores a soul from persistent state.
    ///
    /// # Example
    /// ```
    /// use sanctum_core::{Avatar, PersistentState, RaceType, Gender};
    /// use chrono::Utc;
    /// let state = PersistentState {
    ///     name: "Thrall".to_string(),
    ///     race_type: RaceType::Orc,
    ///     gender: Gender::Male,
    ///     specialization: None,
    ///     xp: 500,
    ///     rested_xp: 0,
    ///     last_saved_at: Utc::now(),
    ///     birth_date: Utc::now(),
    ///     tasks: vec![],
    /// };
    /// let avatar = Avatar::from_state(state);
    /// assert_eq!(avatar.level(), 6);
    /// ```
    pub fn from_state(state: PersistentState) -> Self {
        let mut avatar = Self {
            name: state.name,
            race_type: state.race_type,
            gender: state.gender,
            specialization: state.specialization,
            xp: state.xp,
            rested_xp: state.rested_xp,
            last_saved_at: state.last_saved_at,
            birth_date: state.birth_date,
            satiety: MAX_SATIETY,
            focus: 50.0,
            tasks: state.tasks,
            logs: Vec::new(),
            contributions: Vec::new(),
            last_commit: Utc::now(),
            last_interaction: Utc::now(),
            current_emote: Emote::None,
            emote_end_time: None,
            ultimate_active_until: None,
        };
        avatar.add_log("Hero incarnated. Soul integrity verified.".to_string());
        avatar
    }

    pub fn to_state(&self) -> PersistentState {
        PersistentState {
            name: self.name.clone(),
            race_type: self.race_type,
            gender: self.gender,
            specialization: self.specialization,
            xp: self.xp,
            rested_xp: self.rested_xp,
            last_saved_at: Utc::now(),
            birth_date: self.birth_date,
            tasks: self.tasks.clone(),
        }
    }

    pub fn perform_passive_scan<P: AsRef<Path>>(root: P) -> Vec<Contribution> {
        let mut found = Vec::new();
        if let Ok(walker) = WalkDir::new(root)
            .max_depth(7)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
        {
            for entry in walker {
                let path = entry.path();
                if path.to_string_lossy().contains(".git") && path.ends_with("logs/HEAD") {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        if metadata.is_file() {
                            if let Ok(modified) = metadata.modified() {
                                let project_name = path
                                    .ancestors()
                                    .find(|p| p.ends_with(".git"))
                                    .and_then(|p| p.parent())
                                    .and_then(|p| p.file_name())
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("Unknown Project")
                                    .to_string();
                                if let Some(message) = Self::extract_last_commit_msg(path) {
                                    found.push(Contribution {
                                        project: project_name,
                                        message,
                                        timestamp: modified.into(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        found.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        found
    }

    pub fn extract_last_commit_msg<P: AsRef<Path>>(path: P) -> Option<String> {
        if let Ok(file) = std::fs::File::open(path) {
            let reader = io::BufReader::new(file);
            if let Some(Ok(last_line)) = reader.lines().last() {
                if let Some(msg_part) = last_line.split('\t').next_back() {
                    if last_line.contains('\t') {
                        return Some(msg_part.replace("commit: ", "").trim().to_string());
                    }
                }
            }
        }
        None
    }

    pub fn calculate_offline_gains(&mut self) {
        let now = Utc::now();
        let seconds_away = now.signed_duration_since(self.last_saved_at).num_seconds();
        if seconds_away > 3600 {
            let hours_away = seconds_away / 3600;
            let gained_rested = (hours_away as u32 * 5).min(100);
            self.rested_xp = (self.rested_xp + gained_rested).min(100);
            if gained_rested > 0 {
                self.add_log(format!(
                    "Offline sync complete. +{gained_rested} Rested Bonus recovered."
                ));
            }
        }
    }

    pub fn add_log(&mut self, msg: String) {
        self.logs.push(LogEntry {
            message: msg,
            timestamp: Utc::now(),
        });
        if self.logs.len() > 10 {
            self.logs.remove(0);
        }
    }

    pub fn update_tick(&mut self) {
        let now = Utc::now();
        if let Some(end) = self.emote_end_time {
            if now > end {
                self.current_emote = Emote::None;
                self.emote_end_time = None;
            }
        }
        if let Some(end) = self.ultimate_active_until {
            if now > end {
                self.ultimate_active_until = None;
                self.add_log("System Overclock complete.".to_string());
            }
        }
        let mut decay_rate = SATIETY_DECAY_PER_HOUR;
        if let Some(s) = self.specialization {
            if matches!(
                s,
                Specialization::Paladin | Specialization::DeathKnight | Specialization::Lich
            ) {
                decay_rate = 0.5;
            }
        }
        let hours = now.signed_duration_since(self.last_commit).num_seconds() as f32 / 3600.0;
        self.satiety = (MAX_SATIETY - (hours * decay_rate)).max(0.0);
        let secs_since_inter = now
            .signed_duration_since(self.last_interaction)
            .num_seconds() as f32;
        self.focus = (self.focus - (secs_since_inter / 1800.0 * FOCUS_DECAY_PER_30_MIN)).max(0.0);
    }

    pub fn trigger_emote(&mut self, emote: Emote) {
        self.current_emote = emote;
        self.emote_end_time = Some(Utc::now() + chrono::Duration::seconds(3));
        let msg = self.get_lore_emote_message(emote);
        if !msg.is_empty() {
            self.add_log(format!("{}: {}", self.name, msg));
        }
        self.interact();
    }

    /// ELITE EMOTE MATRIX: Blending canonical personalities with witty development context.
    pub fn get_lore_emote_message(&self, emote: Emote) -> String {
        match (self.name.as_str(), emote) {
            // --- ORC HEROES ---
            ("Thrall", Emote::Cheer) => {
                "I have seen the future, and it is written in Rust. Mostly."
            }
            ("Thrall", Emote::Roar) => "FOR THE HORDE! AND THE PRODUCTION BUILD!",
            ("Thrall", Emote::Dance) => "I am the wielder of the Doomhammer... and the rhythm!",
            ("Thrall", Emote::Salute) => "Winds guide your cursor, Developer.",
            ("Thrall", Emote::Ponder) => {
                "The spirits are restless... likely due to the logic in this PR."
            }
            ("Thrall", Emote::Flex) => "The Earth Mother's strength is in every line of code.",

            ("Grom", Emote::Cheer) => "I can wait no longer... for this CI pipeline!",
            ("Grom", Emote::Roar) => "TASTE MY BLADE, BUGS!",
            ("Grom", Emote::Dance) => "Chaos boils in my veins... and so does the beat!",
            ("Grom", Emote::Salute) => "I serve the Horde... and the stable branch.",
            ("Grom", Emote::Ponder) => "Is it possible? Can this legacy code truly be refactored?",
            ("Grom", Emote::Flex) => "Victory or Death! But preferably a successful build.",

            ("Cairne", Emote::Cheer) => "Your spirit is strong, Developer. The logic holds.",
            ("Cairne", Emote::Salute) => "For the Earth Mother... and the stable branch.",
            ("Cairne", Emote::Ponder) => {
                "Take pride in your work. The ancestors are watching this commit."
            }
            ("Cairne", Emote::Roar) => "ISH-NE-ALO-POR-AH! CRUSH THE ERRORS!",
            ("Cairne", Emote::Dance) => "Ancient rhythm for a modern codebase.",
            ("Cairne", Emote::Flex) => {
                "The strength of the Tauren is in the robustness of the system."
            }

            ("Vol'jin", Emote::Salute) => "I hear da spirits... dey say 'fix da indentation'.",
            ("Vol'jin", Emote::Ponder) => "Who you want me kill? Or just refactor?",
            ("Vol'jin", Emote::Cheer) => "Heh heh... dat logic be powerful magic.",
            ("Vol'jin", Emote::Roar) => "FOR DA DARKSPEAR! AND DA ZERO-DOWNTIME!",
            ("Vol'jin", Emote::Dance) => "Feel da rhythm of da islands... and da async loop.",
            ("Vol'jin", Emote::Flex) => "Cunning beats strength. Optimized code beats all.",

            ("Rexxar", Emote::Salute) => "I track bugs, not animals.",
            ("Rexxar", Emote::Ponder) => {
                "No beast is too wild to debug. Even this legacy monolith."
            }
            ("Rexxar", Emote::Roar) => "FOR THE HORDE! AND FOR CLEAN COMMITS!",
            ("Rexxar", Emote::Cheer) => "My code is my companion. Reliable and well-documented.",
            ("Rexxar", Emote::Dance) => "Dancing with Misha... and the garbage collector.",
            ("Rexxar", Emote::Flex) => "I wander alone, but my PRs are never rejected.",

            ("Rokhan", Emote::Salute) => "I be da Shadow Hunter. Detecting da shadow bugs.",
            ("Rokhan", Emote::Ponder) => "Voodoo logic... it works, don't touch it!",
            ("Rokhan", Emote::Cheer) => "Dat be some good mojo in da repository.",
            ("Rokhan", Emote::Roar) => "TASTE DA VOODOO, DEFECTS!",
            ("Rokhan", Emote::Dance) => "Rhythm of da spirits... syncin' da threads.",
            ("Rokhan", Emote::Flex) => "Shadows protect me. Code reviews respect me.",

            // --- HUMAN HEROES ---
            ("Arthas", Emote::Cheer) => "Justice has come! The integration tests are green.",
            ("Arthas", Emote::Roar) => "I will purge this codebase of all defects!",
            ("Arthas", Emote::Dance) => "I'm a Death Knight Rider! Muh ha ha!",
            ("Arthas", Emote::Salute) => "Glad you could make it, Senior Architect.",
            ("Arthas", Emote::Ponder) => "I hate resorts to monkey-patching. It's so... untidy.",
            ("Arthas", Emote::Flex) => "The Light protects... those who document their code!",

            ("Uther", Emote::Salute) => {
                "Remember, Developer: documentation is the shield of the soul."
            }
            ("Uther", Emote::Ponder) => "I'm too old for these merge conflicts.",
            ("Uther", Emote::Cheer) => "Well done! The Light shines upon this refactor.",
            ("Uther", Emote::Roar) => "FOR LORDAERON! FOR THE TYPE-SAFETY!",
            ("Uther", Emote::Dance) => "A disciplined celebration for a disciplined developer.",
            ("Uther", Emote::Flex) => "Faith in the compiler is the path to victory.",

            ("Jaina", Emote::Cheer) => "Knowledge is power. This algorithm is truly elegant.",
            ("Jaina", Emote::Roar) => "I don't waste magic on just anything. BEGONE BUGS!",
            ("Jaina", Emote::Dance) => "I'm a refined celebration of the academy's linting rules.",
            ("Jaina", Emote::Salute) => "My business is magic... and optimized runtimes.",
            ("Jaina", Emote::Ponder) => {
                "Don't you have a strategy? Pondering the next architectural move."
            }
            ("Jaina", Emote::Flex) => "I've been reading... your documentation. Impressive.",

            ("Antonidas", Emote::Ponder) => "Knowledge is power. But a good cache is faster.",
            ("Antonidas", Emote::Salute) => "You require my assistance? I was mid-compilation.",
            ("Antonidas", Emote::Cheer) => "Remarkable! A most efficient implementation.",
            ("Antonidas", Emote::Roar) => "SILENCE! THE ARCHMAGE IS DEBUGGING!",
            ("Antonidas", Emote::Dance) => "A classic Kirin Tor victory sequence.",
            ("Antonidas", Emote::Flex) => "Age brings wisdom... and very large swap files.",

            ("Muradin", Emote::Roar) => "All right, who wants to be refactored?!",
            ("Muradin", Emote::Cheer) => "That's how we do it in Ironforge! Solid logic!",
            ("Muradin", Emote::Salute) => "Wait 'til you see me in action... on the backend.",
            ("Muradin", Emote::Ponder) => "Is it time for a beer? Or just a code review?",
            ("Muradin", Emote::Dance) => "Watch the beard! It's got its own rhythm!",
            ("Muradin", Emote::Flex) => "Mountain King strength for a scalable architecture!",

            ("Kael'thas", Emote::Roar) => "These bugs are merely a setback!",
            ("Kael'thas", Emote::Ponder) => "My mana is boundless, unlike my stack size.",
            ("Kael'thas", Emote::Cheer) => "Anar'alah belore! By the light of the compiler!",
            ("Kael'thas", Emote::Salute) => "We will reclaim our source code!",
            ("Kael'thas", Emote::Dance) => "Elegant steps for a prince of Quel'Thalas.",
            ("Kael'thas", Emote::Flex) => "The power of the Sunwell flows through my logic.",

            // --- UNDEAD HEROES ---
            ("Sylvanas", Emote::Cheer) => {
                "The code executes... precisely. Victory for the Forsaken."
            }
            ("Sylvanas", Emote::Roar) => "I have no time for manual testing!",
            ("Sylvanas", Emote::Dance) => "Highborne rhythm, even in undeath.",
            ("Sylvanas", Emote::Salute) => "I serve the Queen. And the repository.",
            ("Sylvanas", Emote::Ponder) => "What joy is there in this legacy code? It's a torment.",
            ("Sylvanas", Emote::Flex) => "My arrows fly true. My logic even truer.",

            ("Kel'Thuzad", Emote::Cheer) => {
                "The pact is sealed! Our dominance over the runtime grows."
            }
            ("Kel'Thuzad", Emote::Roar) => "SOULS FOR THE LICH KING! COMMITS FOR THE REPO!",
            ("Kel'Thuzad", Emote::Dance) => "The cold embrace of the dance floor awaits.",
            ("Kel'Thuzad", Emote::Salute) => "I serve the Master... and the master branch.",
            ("Kel'Thuzad", Emote::Ponder) => "The cold embrace of the runtime awaits your bugs.",
            ("Kel'Thuzad", Emote::Flex) => "Witness the power of the Scourge's backend.",

            ("Anub'arak", Emote::Salute) => "From the depths of the legacy system, I serve.",
            ("Anub'arak", Emote::Ponder) => {
                "I'll spin a web of microservices! They'll never escape."
            }
            ("Anub'arak", Emote::Cheer) => "The King comes! The deployment is successful.",
            ("Anub'arak", Emote::Roar) => "CONSUME THE WEAK CODE!",
            ("Anub'arak", Emote::Dance) => "Skittering to the beat of the Frozen Throne.",
            ("Anub'arak", Emote::Flex) => "Indestructible carapace, indestructible logic.",

            ("Mal'Ganis", Emote::Salute) => "I am the darkness... and the root user.",
            ("Mal'Ganis", Emote::Roar) => {
                "I will show you the true meaning of fear... and deadlocks!"
            }
            ("Mal'Ganis", Emote::Ponder) => "Pondering the corruption of the data structures.",
            ("Mal'Ganis", Emote::Cheer) => "Your soul belongs to the repository now.",
            ("Mal'Ganis", Emote::Dance) => "Deceptive rhythm of the Dreadlords.",
            ("Mal'Ganis", Emote::Flex) => "Immortality is mine. As is the production environment.",

            ("Varimathras", Emote::Salute) => "I am a Dreadlord of the Burning Legion. And a SRE.",
            ("Varimathras", Emote::Ponder) => "Betrayal is so... efficient.",
            ("Varimathras", Emote::Cheer) => "Excellent. The plan for dominance is compiling.",
            ("Varimathras", Emote::Roar) => "DIE, BUGS! YOUR TIME IS AT AN END!",
            ("Varimathras", Emote::Dance) => "Dancing on the edge of a stack overflow.",
            ("Varimathras", Emote::Flex) => "Shadow magic and optimized queries.",

            // --- NIGHT ELF HEROES ---
            ("Illidan", Emote::Cheer) => {
                "Ten thousand years of merge conflicts... finally resolved!"
            }
            ("Illidan", Emote::Roar) => "YOU ARE NOT PREPARED... for this merge conflict!",
            ("Illidan", Emote::Dance) => "Chaos boils in my veins... and the bass!",
            ("Illidan", Emote::Salute) => "I am the master of the hunt. And the debug session.",
            ("Illidan", Emote::Ponder) => {
                "Darkness called... I was on a Zoom call, so I missed him."
            }
            ("Illidan", Emote::Flex) => "Now I am complete...ly optimized.",

            ("Tyrande", Emote::Cheer) => "By the light of the moon, the build succeeds!",
            ("Tyrande", Emote::Roar) => "FEAR THE WRATH OF THE GODDESS... AND THE COMPILER!",
            ("Tyrande", Emote::Dance) => "I'm more than a ranger... I'm a night dancer!",
            ("Tyrande", Emote::Salute) => "By the light of Elune, we shall find the memory leak.",
            ("Tyrande", Emote::Ponder) => "The Goddess calls... for better variable naming.",
            ("Tyrande", Emote::Flex) => "My aim is true. My code is thread-safe.",

            ("Malfurion", Emote::Cheer) => {
                "Nature's balance is restored. Zero vulnerabilities found."
            }
            ("Malfurion", Emote::Roar) => "CONSUME THE BUGS! NATURE CALLS!",
            ("Malfurion", Emote::Dance) => "Dancing to the rhythm of the Emerald Dream.",
            ("Malfurion", Emote::Salute) => "Awake. Code is in harmony.",
            ("Malfurion", Emote::Ponder) => {
                "The forest is troubled. Someone ignored the linter again."
            }
            ("Malfurion", Emote::Flex) => "The strength of the Ancients is in this refactor.",

            ("Maiev", Emote::Salute) => "The hunt is on. I will find that rogue thread.",
            ("Maiev", Emote::Cheer) => "Justice will be served... in O(1) time.",
            ("Maiev", Emote::Ponder) => "My quarry is near... just one more stack trace.",
            ("Maiev", Emote::Roar) => "YOU SHALL NOT ESCAPE THE LINTER!",
            ("Maiev", Emote::Dance) => "Swift and deadly rhythm of the Watchers.",
            ("Maiev", Emote::Flex) => "The Warden's duty is eternal. As is the technical debt.",

            ("Akama", Emote::Salute) => {
                "The time for action is now! The time for deployment is sooner!"
            }
            ("Akama", Emote::Ponder) => "My loyalty is to the project, not the legacy system.",
            ("Akama", Emote::Cheer) => "From the ashes of compilation errors, we rise!",
            ("Akama", Emote::Roar) => "WE ARE THE BROKEN, BUT OUR CODE IS WHOLE!",
            ("Akama", Emote::Dance) => "Rhythm of the Outland... syncin' the nodes.",
            ("Akama", Emote::Flex) => {
                "I strike from the shadows. My commits are invisible yet powerful."
            }

            ("Lady Vashj", Emote::Salute) => "For Azshara! For the database schema!",
            ("Lady Vashj", Emote::Ponder) => {
                "Resistance is futile, your merge conflicts are inevitable."
            }
            ("Lady Vashj", Emote::Cheer) => "My power flows like the data stream.",
            ("Lady Vashj", Emote::Roar) => "YOU WILL DROWN IN MY LINES OF CODE!",
            ("Lady Vashj", Emote::Dance) => "Graceful waves and optimized loops.",
            ("Lady Vashj", Emote::Flex) => "Six arms, six concurrent threads.",

            ("Chen", Emote::Salute) => "Share a drink? Or maybe a pull request?",
            ("Chen", Emote::Ponder) => "Another round? Of refactoring, perhaps!",
            ("Chen", Emote::Cheer) => {
                "My code is like a fine brew – it gets better with age and testing."
            }
            ("Chen", Emote::Roar) => "HICCUP IN DA CODE? JUST A MINOR SYNTAX ERROR!",
            ("Chen", Emote::Dance) => "Tipsy rhythm of the Brewmaster.",
            ("Chen", Emote::Flex) => "Strong brew, strong logic, strong deployment.",

            // --- GENERIC FALLBACKS ---
            _ => match (self.race_type, emote) {
                (RaceType::Orc, Emote::Salute) => "Ready for work work.",
                (RaceType::Orc, Emote::Ponder) => "Me no sound like Yoda. Do I?",
                (RaceType::Human, Emote::Salute) => "Orders? Yes, my liege.",
                (RaceType::Human, Emote::Flex) => "The Light protects those who document!",
                (RaceType::Undead, Emote::Dance) => "I'm having a mid-death crisis.",
                (RaceType::Undead, Emote::Ponder) => {
                    "Has Hell frozen over yet? Or just the compiler?"
                }
                (RaceType::NightElf, Emote::Dance) => {
                    "I'm more than a ranger... I'm a night dancer!"
                }
                (RaceType::NightElf, Emote::Roar) => "FEAR MY LEET SKILLS!",
                _ => "Awaiting command...",
            },
        }
        .to_string()
    }

    pub fn use_ultimate(&mut self) {
        if self.focus < 100.0 || self.specialization.is_none() {
            return;
        }
        self.ultimate_active_until = Some(Utc::now() + chrono::Duration::minutes(5));
        self.focus = 0.0;
        let msg = match self.specialization.unwrap() {
            Specialization::Blademaster => "SYSTEM OVERCLOCK: BLADESTORM!",
            Specialization::FarSeer => "SYSTEM OVERCLOCK: FAR SIGHT!",
            Specialization::Paladin => "SYSTEM OVERCLOCK: DIVINE SHIELD!",
            Specialization::Archmage => "SYSTEM OVERCLOCK: BRILLIANCE AURA!",
            Specialization::DeathKnight => "SYSTEM OVERCLOCK: DEATH AND DECAY!",
            Specialization::Lich => "SYSTEM OVERCLOCK: FROST ARMOR!",
            Specialization::DemonHunter => "SYSTEM OVERCLOCK: METAMORPHOSIS!",
            Specialization::Druid => "SYSTEM OVERCLOCK: TRANQUILITY!",
        };
        self.add_log(format!("*** {msg} ***"));
    }

    pub fn interact(&mut self) {
        let mut gain = FOCUS_GAIN_INTERACT;
        if let Some(s) = self.specialization {
            if matches!(
                s,
                Specialization::FarSeer | Specialization::Archmage | Specialization::Lich
            ) {
                gain = 25.0;
            }
        }
        self.focus = (self.focus + gain).min(MAX_FOCUS);
        self.last_interaction = Utc::now();
    }

    pub fn add_task(&mut self, desc: String) {
        self.tasks.push(Task {
            description: desc.clone(),
            completed: false,
        });
        self.add_log(format!("Objective Logged: {desc}"));
        self.interact();
    }
    pub fn toggle_task(&mut self, index: usize) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.completed = !task.completed;
            let d = task.description.clone();
            self.add_log(format!("Objective Updated: {d}"));
            self.interact();
        }
    }
    pub fn remove_task(&mut self, index: usize) {
        if index < self.tasks.len() {
            let t = self.tasks.remove(index);
            self.add_log(format!("Objective Purged: {}", t.description));
            self.interact();
        }
    }

    pub fn link_contribution(&mut self, proj: String, msg: String) {
        let completed = self.tasks.iter().filter(|t| t.completed).count();
        let mut task_bonus = (completed as u32) * 5;
        if matches!(self.specialization, Some(Specialization::Archmage)) {
            task_bonus *= 2;
        }
        let mut threshold = FLOW_STATE_THRESHOLD;
        if matches!(self.specialization, Some(Specialization::DemonHunter)) {
            threshold = 70.0;
        }
        let flow_bonus = if self.focus > threshold { 15 } else { 0 };
        let mut gain = 10 + task_bonus + flow_bonus;
        if self.ultimate_active_until.is_some() {
            gain *= 3;
        }
        if self.rested_xp > 0 {
            let consume = (gain / 2).min(self.rested_xp);
            gain += consume;
            self.rested_xp -= consume;
            self.add_log(format!("Rested bonus: +{consume} XP"));
        }
        let old_lvl = self.level();
        self.xp += gain;
        self.satiety = MAX_SATIETY;
        self.last_commit = Utc::now();
        self.contributions.insert(
            0,
            Contribution {
                project: proj.clone(),
                message: msg,
                timestamp: Utc::now(),
            },
        );
        if self.contributions.len() > 3 {
            self.contributions.pop();
        }
        self.add_log(format!("Contribution Linked: {proj} (+{gain} XP)"));
        if self.level() > old_lvl {
            self.add_log(format!(
                "*** PROMOTION: {} ({}) ***",
                self.level(),
                self.rank()
            ));
        }
    }

    /// Returns the current Hero Level.
    ///
    /// # Example
    /// ```
    /// use sanctum_core::Avatar;
    /// let mut avatar = Avatar::summon();
    /// avatar.xp = 150;
    /// assert_eq!(avatar.level(), 2);
    /// ```
    pub fn level(&self) -> u32 {
        (self.xp / 100) + 1
    }

    pub fn needs_specialization(&self) -> bool {
        self.level() >= SPECIALIZATION_LEVEL && self.specialization.is_none()
    }
    pub fn set_specialization(&mut self, spec: Specialization) {
        self.specialization = Some(spec);
        self.add_log(format!("Career: {spec:?} accepted."));
    }
    pub fn evolution_stage(&self) -> EvolutionStage {
        match self.xp {
            0..100 => EvolutionStage::Egg,
            100..300 => EvolutionStage::Blob,
            300..600 => EvolutionStage::Dog,
            600..1000 => EvolutionStage::Dragon,
            _ => EvolutionStage::Robot,
        }
    }

    /// Returns the current Mood string.
    ///
    /// # Example
    /// ```
    /// use sanctum_core::Avatar;
    /// let avatar = Avatar::summon();
    /// assert_eq!(avatar.mood(), "Operational");
    /// ```
    pub fn mood(&self) -> String {
        if self.ultimate_active_until.is_some() {
            return "OVERCLOCKED".to_string();
        }
        if self.satiety < 20.0 {
            return "System Idle".to_string();
        }
        let threshold = if matches!(self.specialization, Some(Specialization::DemonHunter)) {
            70.0
        } else {
            FLOW_STATE_THRESHOLD
        };
        if self.focus > threshold {
            return "Synchronized".to_string();
        }
        "Operational".to_string()
    }

    pub fn rank(&self) -> String {
        if let Some(spec) = self.specialization {
            return format!("{spec:?}");
        }
        let l = self.level();
        match self.race_type {
            RaceType::Orc => match l {
                1 => "Grunt",
                2 => "Raider",
                3 => "Sergeant",
                4 => "Senior Sergeant",
                5 => "First Sergeant",
                6 => "Stone Guard",
                7 => "Blood Guard",
                8 => "Legionnaire",
                9 => "Centurion",
                _ => "General",
            },
            RaceType::Human => match l {
                1 => "Footman",
                2 => "Knight",
                3 => "Corporal",
                4 => "Master Sergeant",
                5 => "Sergeant Major",
                6 => "Knight-Lieutenant",
                7 => "Knight-Captain",
                8 => "Knight-Champion",
                9 => "Marshal",
                _ => "Commander",
            },
            RaceType::Undead => match l {
                1 => "Ghoul",
                2 => "Stalker",
                3 => "Deathguard",
                4 => "Shadow Guard",
                5 => "Sentinel",
                6 => "Champion",
                7 => "Overlord",
                8 => "Death Lord",
                9 => "Baron",
                _ => "Dreadlord",
            },
            RaceType::NightElf => match l {
                1 => "Archer",
                2 => "Huntress",
                3 => "Sentinel",
                4 => "Outrunner",
                5 => "Watcher",
                6 => "Guardian",
                7 => "Keeper",
                8 => "Ancient",
                9 => "High Sentinel",
                _ => "Arch-Sentinel",
            },
        }
        .to_string()
    }

    pub fn get_comment(&self) -> String {
        if self.ultimate_active_until.is_some() {
            return "LIMIT BREAKER: PROCESSING.".to_string();
        }
        if self.satiety < 20.0 {
            return match self.race_type {
                RaceType::Orc => "Work work? Me fix bugs?",
                RaceType::Human => "The Compiler demands documentation.",
                RaceType::Undead => "We never truly die... like legacy code.",
                RaceType::NightElf => "Ten thousand years of merge conflicts.",
            }
            .to_string();
        }
        if let Some(t) = self.tasks.iter().find(|t| !t.completed) {
            return format!("Objective: {}", t.description);
        }
        "Awaiting tactical assignment.".to_string()
    }

    pub fn get_portrait_layers(&self) -> Vec<String> {
        let sec = Utc::now().second();
        let is_blink = sec % 4 == 0;
        let is_female = matches!(self.gender, Gender::Female);
        let mut layers = vec![
            "         ".to_string(),
            "  .---.  ".to_string(),
            " / o o \\ ".to_string(),
            "(   -   )".to_string(),
            " \\  -- / ".to_string(),
            "  `---'  ".to_string(),
        ];
        match self.race_type {
            RaceType::Orc => {
                layers[0] = "  ^   ^  ".to_string();
                layers[2] = if is_blink {
                    " / - - \\ ".to_string()
                } else {
                    " / O O \\ ".to_string()
                };
                layers[3] = "(   ^   )".to_string();
                layers[4] = " \\ === / ".to_string();
            }
            RaceType::Human => {
                layers[0] = if is_female {
                    "  ,,,,,  ".to_string()
                } else {
                    "  _____  ".to_string()
                };
                layers[2] = if is_blink {
                    " / - - \\ ".to_string()
                } else {
                    " / ^ ^ \\ ".to_string()
                };
            }
            RaceType::Undead => {
                layers[0] = "  ~ ~ ~  ".to_string();
                layers[2] = " / X X \\ ".to_string();
                layers[3] = "(   ~   )".to_string();
                layers[4] = " \\ ~~~ / ".to_string();
            }
            RaceType::NightElf => {
                layers[0] = " /     \\ ".to_string();
                layers[2] = if is_blink {
                    " / - - \\ ".to_string()
                } else {
                    " / * * \\ ".to_string()
                };
                layers[3] = "(   .   )".to_string();
            }
        }
        if let Some(s) = self.specialization {
            match s {
                Specialization::Blademaster => {
                    layers[0] = "  /|||\\  ".to_string();
                    layers[3] = "(  [^]  )".to_string();
                }
                Specialization::Archmage => {
                    layers[0] = "  .+++.  ".to_string();
                }
                Specialization::Lich => {
                    layers[5] = "  <###>  ".to_string();
                }
                Specialization::DemonHunter => {
                    layers[2] = " / ╬ ╬ \\ ".to_string();
                }
                Specialization::Paladin => {
                    layers[1] = " .[###]. ".to_string();
                }
                _ => {}
            }
        }
        match self.current_emote {
            Emote::Cheer => layers[4] = "  \\___/  ".to_string(),
            Emote::Roar => layers[4] = "  ( O )  ".to_string(),
            Emote::Ponder => layers[2] = " / ? ? \\ ".to_string(),
            Emote::Flex => {
                layers[3] = " <( ^ )> ".to_string();
                layers[4] = "  | |  ".to_string();
            }
            Emote::Dance => {
                if sec % 2 == 0 {
                    layers[4] = "  / - \\  ".to_string();
                }
            }
            _ => {}
        }
        layers
    }
}

// --- TUI Logic ---
pub const APP_VERSION: &str = "v2.9.5-ULTIMATE";
pub const TICK_RATE: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    Specializing,
    Emoting,
    SelectingProfile,
    CreatingProfile,
    ConfirmingDelete(String),
}

pub struct App {
    pub avatar: Avatar,
    pub input: String,
    pub input_mode: InputMode,
    pub list_state: ListState,
    pub profile_list: Vec<String>,
    pub profile_list_state: ListState,
}

impl App {
    pub fn new(avatar: Avatar) -> Self {
        let mut list_state = ListState::default();
        if !avatar.tasks.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            avatar,
            input: String::new(),
            input_mode: InputMode::Normal,
            list_state,
            profile_list: Vec::new(),
            profile_list_state: ListState::default(),
        }
    }
    pub fn next_task(&mut self) {
        if self.avatar.tasks.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.avatar.tasks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
    pub fn previous_task(&mut self) {
        if self.avatar.tasks.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.avatar.tasks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
    pub fn refresh_profiles(&mut self) {
        self.profile_list = list_profiles();
        if self.profile_list.is_empty() {
            self.profile_list_state.select(None);
        } else {
            let next = self
                .profile_list_state
                .selected()
                .unwrap_or(0)
                .min(self.profile_list.len() - 1);
            self.profile_list_state.select(Some(next));
        }
    }
}

pub fn get_data_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "adity", "terminal-sanctum").map(|p| {
        let d = p.data_dir().join("profiles");
        std::fs::create_dir_all(&d).ok();
        d
    })
}

pub fn list_profiles() -> Vec<String> {
    let mut profiles = Vec::new();
    if let Some(dir) = get_data_dir() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_file() && entry.path().extension().is_some_and(|e| e == "json") {
                        if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                            profiles.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    profiles.sort();
    profiles
}

pub fn save_avatar(a: &Avatar) -> io::Result<()> {
    if let Some(mut path) = get_data_dir() {
        path.push(format!("{}.json", a.name));
        let state = a.to_state();
        let j = serde_json::to_string(&state).map_err(io::Error::other)?;
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, j)?;
        std::fs::rename(tmp_path, path)
    } else {
        Err(io::Error::other("Could not resolve data directory"))
    }
}

pub fn load_avatar(name: &str) -> io::Result<Avatar> {
    if let Some(mut path) = get_data_dir() {
        path.push(format!("{name}.json"));
        let data = std::fs::read_to_string(path)?;
        let state: PersistentState = serde_json::from_str(&data).map_err(io::Error::other)?;
        Ok(Avatar::from_state(state))
    } else {
        Err(io::Error::other("Could not resolve data directory"))
    }
}

pub fn ui(f: &mut ratatui::Frame, app: &mut App) {
    if matches!(app.input_mode, InputMode::SelectingProfile) {
        render_profile_selection(f, app);
        return;
    }
    if matches!(app.input_mode, InputMode::CreatingProfile) {
        render_profile_creation(f, app);
        return;
    }
    if let InputMode::ConfirmingDelete(name) = &app.input_mode {
        render_delete_confirmation(f, name);
        return;
    }
    let root = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(12),
                Constraint::Min(5),
                Constraint::Length(7),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());
    let dashboard = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(25), Constraint::Min(20)].as_ref())
        .split(root[0]);
    render_portrait_box(f, app, dashboard[0]);
    render_stats_panel(f, app, dashboard[1]);
    render_task_list(f, app, root[1]);
    render_activity_log(f, app, root[2]);
    pub fn render_controls(f: &mut ratatui::Frame, app: &App, area: Rect) {
        let msg = match app.input_mode {
            InputMode::Normal => "(a) Objective | (e) Emote | (u) Overclock | (s) Switch | (Space) Resolve | (x) Delete | (q) Quit".to_string(),
            InputMode::Editing => format!("Define Objective: {}_ (Enter to Accept)", app.input),
            InputMode::Specializing => "CHOOSE CAREER PATH (Press 1 or 2)".to_string(),
            InputMode::Emoting => "SELECT EMOTE (A-H) or ESC".to_string(),
            _ => "".to_string(),
        };
        f.render_widget(
            Paragraph::new(msg).alignment(Alignment::Center).block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Yellow)),
            ),
            area,
        );
    }
    render_controls(f, app, root[3]);
    if matches!(app.input_mode, InputMode::Specializing) {
        render_specialization_modal(f, app);
    }
    if matches!(app.input_mode, InputMode::Emoting) {
        render_emote_modal(f);
    }
}

fn render_profile_selection(f: &mut ratatui::Frame, app: &mut App) {
    let area = centered_rect(70, 70, f.size());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" 🏛️  HALL OF HEROES ({APP_VERSION})  🏛️  "));
    f.render_widget(block, area);
    let inner = centered_rect(60, 50, f.size());
    let items: Vec<ListItem> = app
        .profile_list
        .iter()
        .map(|p| ListItem::new(format!("  🛡️  {p}")))
        .collect();
    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ▶ ");
    let help =
        Paragraph::new("(j/k) Scroll | (Enter) Command | (c) Summon | (d) Delete | (q) Exit")
            .alignment(Alignment::Center);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(inner);
    f.render_stateful_widget(list, chunks[0], &mut app.profile_list_state);
    f.render_widget(help, chunks[1]);
}

fn render_delete_confirmation(f: &mut ratatui::Frame, name: &str) {
    let area = centered_rect(65, 35, f.size());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" ⚠️  DELETE CHARACTER CONFIRMATION  ⚠️ ");
    let inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    let msg = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("Are you certain you wish to delete {name}?"),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "This will PERMANENTLY delete all progress, levels,",
            Style::default().fg(Color::Gray),
        )]),
        Line::from(vec![Span::styled(
            "and objectives for this character avatar profile.",
            Style::default().fg(Color::Gray),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "(y) Yes, Delete",
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  |  "),
            Span::styled(
                "(n) No, Cancel",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    f.render_widget(
        Paragraph::new(msg)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        inner,
    );
}

fn render_profile_creation(f: &mut ratatui::Frame, _app: &App) {
    let area = centered_rect(50, 30, f.size());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ⚔️  DIVINE SUMMONING  ⚔️ ");
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    let inner = centered_rect(40, 15, area);
    f.render_widget(
        Paragraph::new(
            "A legendary soul awaits your call...\n\n(Enter) Summon Random Hero | (Esc) Cancel",
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow)),
        inner,
    );
}

fn render_portrait_box(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let race = match app.avatar.race_type {
        RaceType::Orc => "ORC",
        RaceType::Human => "HUMAN",
        RaceType::Undead => "UNDEAD",
        RaceType::NightElf => "NIGHT ELF",
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {race} "));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let lines = app.avatar.get_portrait_layers();
    let mut rendered = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        let color = match app.avatar.race_type {
            RaceType::Orc => Color::Rgb(0, (255 - i * 30).max(0) as u8, 50),
            RaceType::Human => Color::Rgb(
                (200 + i * 10).min(255) as u8,
                (200 + i * 10).min(255) as u8,
                (255 - i * 20).max(0) as u8,
            ),
            RaceType::Undead => {
                Color::Rgb((150 - i * 20).max(0) as u8, 0, (200 - i * 20).max(0) as u8)
            }
            RaceType::NightElf => Color::Rgb(
                (100 + i * 20).min(255) as u8,
                (100 - i * 10).max(0) as u8,
                255,
            ),
        };
        rendered.push(Line::from(Span::styled(
            l.clone(),
            Style::default().fg(color),
        )));
    }
    f.render_widget(Paragraph::new(rendered).alignment(Alignment::Center), inner);
}

fn render_stats_panel(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)].as_ref())
        .split(area);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(main[0]);
    let gender = match app.avatar.gender {
        Gender::Male => "♂",
        Gender::Female => "♀",
    };
    let name_line = Line::from(vec![
        Span::styled(
            format!(" {} ", app.avatar.name),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        ),
        Span::styled(
            format!(" {gender} "),
            Style::default().fg(if gender == "♂" {
                Color::Blue
            } else {
                Color::LightRed
            }),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Lvl {} ", app.avatar.level()),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            format!("[{}]", app.avatar.rank()),
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        ),
    ]);
    f.render_widget(Paragraph::new(name_line), left[0]);
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(Color::LightGreen))
            .percent(app.avatar.satiety as u16)
            .label(Span::styled(
                "COMMIT ENERGY",
                Style::default()
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
        left[1],
    );
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(app.avatar.focus as u16)
            .label(Span::styled(
                format!("FOCUS (+{} RESTED)", app.avatar.rested_xp),
                Style::default()
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
        left[2],
    );
    let speech = Paragraph::new(format!(
        "\nSTATUS: {}\n\"{}\"",
        app.avatar.mood(),
        app.avatar.get_comment()
    ))
    .wrap(Wrap { trim: true })
    .style(Style::default().fg(Color::Gray));
    f.render_widget(speech, left[3]);
    let history: Vec<ListItem> = app
        .avatar
        .contributions
        .iter()
        .map(|c| {
            ListItem::new(vec![
                Line::from(vec![Span::styled(
                    format!("• {}", c.project),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(Span::styled(
                    format!("  \"{}\"", c.message),
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                )),
            ])
        })
        .collect();
    f.render_widget(
        List::new(history).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" RECENT CONTRIBUTIONS "),
        ),
        main[1],
    );
}

fn render_task_list(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .avatar
        .tasks
        .iter()
        .map(|t| {
            let (sym, style) = if t.completed {
                (
                    "✓",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::CROSSED_OUT),
                )
            } else {
                ("☐", Style::default().fg(Color::White))
            };
            ListItem::new(format!("  {}  {}", sym, t.description)).style(style)
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" OBJECTIVE LOG "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 40, 40))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ▶ ");
    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_activity_log(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let logs: Vec<String> = app
        .avatar
        .logs
        .iter()
        .rev()
        .map(|l| format!("[{}] {}", l.timestamp.format("%H:%M:%S"), l.message))
        .collect();
    f.render_widget(
        Paragraph::new(logs.join("\n"))
            .block(Block::default().borders(Borders::ALL).title(" SYSTEM LOG "))
            .style(Style::default().fg(Color::Gray)),
        area,
    );
}

fn render_specialization_modal(f: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(60, 40, f.size());
    f.render_widget(Clear, area);
    let (o1, o2) = match app.avatar.race_type {
        RaceType::Orc => ("1. Blademaster", "2. Far Seer"),
        RaceType::Human => ("1. Paladin", "2. Archmage"),
        RaceType::Undead => ("1. Death Knight", "2. Lich"),
        RaceType::NightElf => ("1. Demon Hunter", "2. Druid"),
    };
    let text = vec![
        Line::from(Span::styled(
            "--- PROFESSIONAL ASCENSION ---",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![Span::styled(o1, Style::default().fg(Color::Cyan))]),
        Line::from(vec![Span::styled(o2, Style::default().fg(Color::Cyan))]),
    ];
    f.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .alignment(Alignment::Center),
        area,
    );
}

fn render_emote_modal(f: &mut ratatui::Frame) {
    let area = centered_rect(45, 45, f.size());
    let block = Block::default()
        .title(" EMOTES ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let diagram = vec![
        Line::from(""),
        Line::from("      [ E ]      "),
        Line::from("        |        "),
        Line::from("  [A][S][D][F][G][H]"),
        Line::from(""),
        Line::from(vec![
            Span::styled("A: ", Style::default().fg(Color::Yellow)),
            Span::raw("Cheer  "),
            Span::styled("S: ", Style::default().fg(Color::Yellow)),
            Span::raw("Roar"),
        ]),
        Line::from(vec![
            Span::styled("D: ", Style::default().fg(Color::Yellow)),
            Span::raw("Dance  "),
            Span::styled("F: ", Style::default().fg(Color::Yellow)),
            Span::raw("Salute"),
        ]),
        Line::from(vec![
            Span::styled("G: ", Style::default().fg(Color::Yellow)),
            Span::raw("Ponder "),
            Span::styled("H: ", Style::default().fg(Color::Yellow)),
            Span::raw("Flex"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "(Esc) Cancel",
            Style::default().fg(Color::Gray),
        )),
    ];
    f.render_widget(Paragraph::new(diagram).alignment(Alignment::Center), inner);
}

fn centered_rect(px: u16, py: u16, r: Rect) -> Rect {
    let vl = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - py) / 2),
                Constraint::Percentage(py),
                Constraint::Percentage((100 - py) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - px) / 2),
                Constraint::Percentage(px),
                Constraint::Percentage((100 - px) / 2),
            ]
            .as_ref(),
        )
        .split(vl[1])[1]
}

fn get_legendary_name(r: RaceType, g: Gender, rng: &mut rand::rngs::ThreadRng) -> String {
    let n = match (r, g) {
        (RaceType::Orc, Gender::Male) => vec![
            "Thrall",
            "Grom",
            "Cairne",
            "Vol'jin",
            "Rexxar",
            "Rokhan",
            "Drek'Thar",
            "Nazgrel",
        ],
        (RaceType::Orc, Gender::Female) => vec!["Draka", "Aggra", "Garona", "Zaela"],
        (RaceType::Human, Gender::Male) => vec![
            "Arthas",
            "Uther",
            "Muradin",
            "Antonidas",
            "Anduin",
            "Kael'thas",
            "Turalyon",
            "Khadgar",
            "Genn",
        ],
        (RaceType::Human, Gender::Female) => vec!["Jaina", "Modera", "Calia", "Alleria", "Vereesa"],
        (RaceType::Undead, Gender::Male) => vec![
            "Kel'Thuzad",
            "Anub'arak",
            "Mal'Ganis",
            "Varimathras",
            "Tichondrius",
            "Putress",
            "Nathanos",
            "Balnazzar",
        ],
        (RaceType::Undead, Gender::Female) => vec!["Sylvanas", "Faerlina", "Anastari", "Lilian"],
        (RaceType::NightElf, Gender::Male) => vec![
            "Malfurion",
            "Illidan",
            "Cenarius",
            "Jarod",
            "Akama",
            "Broll",
        ],
        (RaceType::NightElf, Gender::Female) => vec![
            "Tyrande",
            "Maiev",
            "Shandris",
            "Naisha",
            "Lady Vashj",
            "Lunara",
        ],
    };
    n[rng.gen_range(0..n.len())].to_string()
}

fn get_intro_message(r: RaceType, name: &str) -> String {
    match name {
        "Thrall" => "I have seen the future, and it is written in Rust. Mostly.",
        "Arthas" => "I will purge this codebase of all defects!",
        "Illidan" => "YOU ARE NOT PREPARED... for this merge conflict!",
        "Sylvanas" => "I have no time for manual testing!",
        "Kael'thas" => "Anar'alah belore! By the light of the compiler!",
        "Rexxar" => "I track bugs, not animals.",
        "Chen" => "Another round? Of refactoring, perhaps!",
        "Akama" => "We are the Broken, but our code is whole.",
        "Antonidas" => "Knowledge is power. But a good cache is faster.",
        _ => match r {
            RaceType::Orc => "The spirits are restless... likely due to the logic in this PR.",
            RaceType::Human => "Justice has come! Systems online.",
            RaceType::Undead => "The shadows recede... legacy refactoring active.",
            RaceType::NightElf => "Awake. Nature's balance is in harmony.",
        },
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_xp() {
        let mut a = Avatar::summon();
        a.xp = 100;
        assert_eq!(a.level(), 2);
    }
    #[test]
    fn test_ranks() {
        let mut a = Avatar::summon();
        a.race_type = RaceType::Orc;
        assert_eq!(a.rank(), "Grunt");
    }
    #[test]
    fn test_pers() {
        let mut a = Avatar::summon();
        a.add_task("T".to_string());
        let s = a.to_state();
        let r = Avatar::from_state(s);
        assert_eq!(r.tasks.len(), 1);
    }
}
