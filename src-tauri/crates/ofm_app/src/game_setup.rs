//! Game setup and bootstrap logic shared between the Tauri desktop frontend
//! and the handheld frontend. Extracted from `src-tauri/src/commands/game.rs`
//! — every function here has zero Tauri dependencies.

use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use domain::league::{
    CompetitionFormat, CompetitionScope, CompetitionType, FixtureCompetition, League,
};
use domain::manager::Manager;
use domain::national_team::NationalTeam;
use domain::stats::StatsState;
use ofm_core::clock::GameClock;
use ofm_core::game::Game;

// ── Constants ───────────────────────────────────────────────────────

pub const DEFAULT_GENERATED_HISTORY_DEPTH_YEARS: u32 = 12;
pub const MAX_GENERATED_HISTORY_DEPTH_YEARS: u32 = 24;

const TOP_DIVISION_SIZE: usize = 20;
const CONTINENTAL_CHAMPIONS_CUP_ID: &str = "continental-champions-cup";
const CONTINENTAL_QUALIFYING_POSITIONS: u32 = 4;
const PRESEASON_ANCHOR_BUFFER_DAYS: i64 = 30;

// ── Types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawStartupOptions {
    #[serde(default)]
    pub start_year: Option<i32>,
    #[serde(default)]
    pub start_phase: Option<String>,
    #[serde(default)]
    pub history_depth_years: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartPhase {
    SeasonStart,
    MidSeason,
}

impl StartPhase {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "seasonStart" => Some(Self::SeasonStart),
            "midSeason" => Some(Self::MidSeason),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SeasonStart => "seasonStart",
            Self::MidSeason => "midSeason",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupOptions {
    pub start_year: i32,
    pub start_phase: StartPhase,
    pub history_depth_years: u32,
}

// ── Small helpers ───────────────────────────────────────────────────

pub fn long_date_format() -> String {
    ['%', 'B', ' ', '%', 'd', ',', ' ', '%', 'Y']
        .into_iter()
        .collect()
}

pub fn default_league_name() -> String {
    ["Premier", "Division"].join(" ")
}

pub fn default_start_year() -> i32 {
    chrono::Utc::now().year().max(2020)
}

pub fn default_history_depth_years() -> u32 {
    DEFAULT_GENERATED_HISTORY_DEPTH_YEARS
}

pub fn start_date_for_year(start_year: i32) -> Result<DateTime<Utc>, String> {
    let month = if ofm_core::world_cup::is_world_cup_summer(start_year) {
        6
    } else {
        7
    };
    Utc.with_ymd_and_hms(start_year, month, 1, 0, 0, 0)
        .single()
        .ok_or_else(|| "be.error.createManager.invalidStartYear".to_string())
}

pub fn current_date_for_phase(
    start_year: i32,
    start_phase: StartPhase,
) -> Result<DateTime<Utc>, String> {
    let start_date = start_date_for_year(start_year)?;
    Ok(match start_phase {
        StartPhase::SeasonStart => start_date,
        StartPhase::MidSeason => start_date + Duration::days(120),
    })
}

pub fn age_on_date(birth_date: chrono::NaiveDate, reference_date: chrono::NaiveDate) -> i64 {
    let mut age = i64::from(reference_date.year() - birth_date.year());
    let has_had_birthday =
        (reference_date.month(), reference_date.day()) >= (birth_date.month(), birth_date.day());
    if !has_had_birthday {
        age -= 1;
    }
    age
}

pub fn normalize_startup_options(raw: Option<RawStartupOptions>) -> Result<StartupOptions, String> {
    let raw = raw.unwrap_or_default();
    let start_year = raw.start_year.unwrap_or_else(default_start_year);
    if start_year < 2020 {
        return Err("be.error.createManager.startYearMin".to_string());
    }
    let start_phase = match raw.start_phase.as_deref() {
        None | Some("") => StartPhase::SeasonStart,
        Some(value) => StartPhase::parse(value)
            .ok_or_else(|| "be.error.createManager.invalidStartPhase".to_string())?,
    };
    let history_depth_years = raw
        .history_depth_years
        .unwrap_or_else(default_history_depth_years);
    if history_depth_years > MAX_GENERATED_HISTORY_DEPTH_YEARS {
        return Err("be.error.createManager.historyDepthMax".to_string());
    }
    Ok(StartupOptions {
        start_year,
        start_phase,
        history_depth_years,
    })
}

pub fn start_phase_for_game(game: &Game) -> StartPhase {
    if game.clock.current_date > game.clock.start_date {
        StartPhase::MidSeason
    } else {
        StartPhase::SeasonStart
    }
}

pub fn preseason_season_start(clock: &GameClock) -> DateTime<Utc> {
    clock.start_date + Duration::days(30)
}

pub fn preseason_league_year(clock: &GameClock) -> u32 {
    let year = clock.start_date.year() + i32::from(clock.start_date.month() == 12);
    u32::try_from(year).unwrap_or(2020)
}

// ── Region / country helpers ────────────────────────────────────────

pub fn infer_region_id(country_code: &str) -> String {
    ofm_core::nations::region_for_code(country_code).to_string()
}

pub fn infer_team_region_id(team: &domain::team::Team) -> String {
    if !team.football_nation.is_empty() {
        return infer_region_id(&team.football_nation);
    }
    infer_region_id(&team.country)
}

pub fn competition_required_region_ids(competition: &League) -> Vec<String> {
    let mut region_ids = competition.required_region_ids.clone();
    if matches!(
        competition.scope,
        CompetitionScope::Domestic | CompetitionScope::Regional
    ) {
        if let Some(region_id) = &competition.region_id {
            region_ids.push(region_id.clone());
        }
    }
    region_ids.sort();
    region_ids.dedup();
    region_ids
}

pub fn default_season_month_for_region(region_id: &str) -> u8 {
    match region_id {
        "south-america" => 3,
        "asia" => 2,
        "oceania" => 10,
        _ => 8,
    }
}

pub fn brazil_state_region(city: &str) -> Option<&'static str> {
    match city {
        "São Paulo" | "Rio" | "Belo Horizonte" | "Santos" | "Campinas" | "Bragantino"
        | "Juiz de Fora" | "Vitória" => Some("southeast"),
        "Porto Alegre" | "Curitiba" | "Florianópolis" => Some("south"),
        "Salvador" | "Recife" | "Fortaleza" | "Natal" | "Maceió" => Some("northeast"),
        "Goiânia" | "Belém" | "Manaus" | "Cuiabá" => Some("north-central-west"),
        _ => None,
    }
}

// ── Division helpers ────────────────────────────────────────────────

pub fn split_into_divisions(sorted_team_ids: &[String], division_size: usize) -> Vec<Vec<String>> {
    let division_size = division_size.max(2);
    if sorted_team_ids.len() <= division_size {
        return vec![sorted_team_ids.to_vec()];
    }
    let mut divisions: Vec<Vec<String>> = sorted_team_ids
        .chunks(division_size)
        .map(<[String]>::to_vec)
        .collect();
    if divisions.len() >= 2 && divisions.last().map(Vec::len).unwrap_or(0) < division_size / 2 {
        let tail = divisions.pop().expect("len >= 2");
        divisions.last_mut().expect("len >= 1").extend(tail);
    }
    divisions
}

pub fn division_tier_name(tier: usize, division_count: usize) -> &'static str {
    if division_count <= 1 {
        "League"
    } else if tier == 0 {
        "First Division"
    } else {
        "Second Division"
    }
}

pub fn division_tier_name_key(tier: usize, division_count: usize) -> &'static str {
    if division_count <= 1 {
        "tournaments.competitions.league"
    } else if tier == 0 {
        "tournaments.competitions.firstDivision"
    } else {
        "tournaments.competitions.secondDivision"
    }
}

pub fn division_name(country: &str, tier: usize, division_count: usize) -> String {
    format!("{country} {}", division_tier_name(tier, division_count))
}

// ── National teams ──────────────────────────────────────────────────

pub fn build_national_teams(game: &Game) -> Vec<NationalTeam> {
    use std::collections::BTreeMap;

    let mut players_by_nation: BTreeMap<String, Vec<&domain::player::Player>> = BTreeMap::new();
    for player in &game.players {
        let nation = if player.football_nation.is_empty() {
            player.nationality.clone()
        } else {
            player.football_nation.clone()
        };
        players_by_nation.entry(nation).or_default().push(player);
    }

    players_by_nation
        .into_iter()
        .map(|(nation, mut players)| {
            players.sort_by(|left, right| right.ovr.cmp(&left.ovr));
            let nation_label = ofm_core::nations::nation_display_name(&nation);
            let mut national_team = NationalTeam::new(
                format!("nt-{}", nation.to_lowercase()),
                format!("{} National Team", nation_label),
                nation.clone(),
                Some(game.region_for_country(&nation)),
            );
            national_team.squad_player_ids = players
                .into_iter()
                .take(23)
                .map(|player| player.id.clone())
                .collect();
            national_team
        })
        .collect()
}

// ── Continental entrants ────────────────────────────────────────────

pub fn select_continental_entrants(
    teams: &[domain::team::Team],
    per_region: usize,
    max_entrants: usize,
) -> Vec<String> {
    use std::collections::BTreeMap;

    let reputation_then_id = |left: &&domain::team::Team, right: &&domain::team::Team| {
        right
            .reputation
            .cmp(&left.reputation)
            .then_with(|| left.id.cmp(&right.id))
    };

    let mut teams_by_region: BTreeMap<String, Vec<&domain::team::Team>> = BTreeMap::new();
    for team in teams {
        teams_by_region
            .entry(infer_team_region_id(team))
            .or_default()
            .push(team);
    }

    let mut entrants: Vec<&domain::team::Team> = Vec::new();
    for regional_teams in teams_by_region.values_mut() {
        regional_teams.sort_by(reputation_then_id);
        entrants.extend(regional_teams.iter().take(per_region).copied());
    }

    entrants.sort_by(reputation_then_id);
    entrants
        .into_iter()
        .take(max_entrants)
        .map(|team| team.id.clone())
        .collect()
}

// ── Foundation competition plan ─────────────────────────────────────

pub fn build_foundation_competition_plan(
    game: &Game,
    game_start: DateTime<Utc>,
) -> Vec<(ofm_core::generator::CompetitionDefinition, DateTime<Utc>)> {
    use domain::league::{Berth, BerthRule};
    use ofm_core::generator::{CompetitionDefinition, FormatDef, ParticipantSpec};
    use std::collections::BTreeMap;

    let continental_berth = |rule: BerthRule| Berth {
        target: CONTINENTAL_CHAMPIONS_CUP_ID.to_string(),
        rule,
        fallback_to: None,
    };

    let make_format = |kind: CompetitionFormat| FormatDef {
        kind,
        legs: None,
        group_size: None,
        qualifiers_per_group: None,
        best_third_qualifiers: None,
    };

    let mut teams_by_country: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for team in &game.teams {
        teams_by_country
            .entry(team.football_nation.clone())
            .or_default()
            .push(team.id.clone());
    }

    let reputation: std::collections::HashMap<&str, u32> = game
        .teams
        .iter()
        .map(|team| (team.id.as_str(), team.reputation))
        .collect();

    let mut planned: Vec<(CompetitionDefinition, DateTime<Utc>)> = Vec::new();
    let mut priority = 0u32;
    for (country, mut team_ids) in teams_by_country {
        if team_ids.len() < 2 {
            continue;
        }
        team_ids.sort_by(|left, right| {
            reputation
                .get(right.as_str())
                .cmp(&reputation.get(left.as_str()))
                .then_with(|| left.cmp(right))
        });
        let region_id = infer_region_id(&country);
        let country_label = ofm_core::nations::nation_display_name(&country);
        let country_slug = country.to_lowercase();

        let league_month = if country == "BR" {
            1
        } else {
            default_season_month_for_region(&region_id)
        };
        let (league_start, _) = ofm_core::generator::start_date_at_game_open(
            game_start,
            league_month,
            if country == "BR" { 28 } else { 1 },
        );

        let divisions = split_into_divisions(&team_ids, TOP_DIVISION_SIZE);
        let division_count = divisions.len();

        if ofm_core::nations::is_split_season_country(&country) {
            let (apertura_start, _) =
                ofm_core::generator::start_date_at_game_open(game_start, 2, 1);
            let (clausura_start, _) =
                ofm_core::generator::start_date_at_game_open(game_start, 7, 1);

            for (tier, division_ids) in divisions.iter().enumerate() {
                let clausura_berths = if tier == 0 {
                    vec![continental_berth(BerthRule::PositionRange {
                        from: 1,
                        to: CONTINENTAL_QUALIFYING_POSITIONS,
                    })]
                } else {
                    Vec::new()
                };
                let make_def = |id: &str, name: &str, month: u8, berths: Vec<Berth>, p: u32| {
                    CompetitionDefinition {
                        id: id.to_string(),
                        name: name.to_string(),
                        r#type: CompetitionType::League,
                        scope: CompetitionScope::Domestic,
                        region_id: Some(region_id.clone()),
                        country_id: Some(country.clone()),
                        required_region_ids: vec![region_id.clone()],
                        priority: p,
                        format: make_format(CompetitionFormat::LeagueTable),
                        participants: ParticipantSpec {
                            explicit: Some(division_ids.clone()),
                            selector: None,
                        },
                        berths,
                        season_start_month: Some(month),
                        season_start_day: Some(1),
                        name_key: None,
                        logo: None,
                    }
                };
                let tier_suffix = format!("d{}", tier + 1);
                planned.push((
                    make_def(
                        &format!("{country_slug}-{tier_suffix}-apertura"),
                        &format!(
                            "{country_label} {} Apertura",
                            division_tier_name(tier, division_count)
                        ),
                        2,
                        Vec::new(),
                        priority,
                    ),
                    apertura_start,
                ));
                priority += 1;
                planned.push((
                    make_def(
                        &format!("{country_slug}-{tier_suffix}-clausura"),
                        &format!(
                            "{country_label} {} Clausura",
                            division_tier_name(tier, division_count)
                        ),
                        7,
                        clausura_berths,
                        priority,
                    ),
                    clausura_start,
                ));
                priority += 1;
            }
        } else {
            for (tier, division_ids) in divisions.iter().enumerate() {
                let berths = if tier == 0 {
                    vec![continental_berth(BerthRule::PositionRange {
                        from: 1,
                        to: CONTINENTAL_QUALIFYING_POSITIONS,
                    })]
                } else {
                    Vec::new()
                };
                let actual_start = if country == "BR" && tier > 0 {
                    ofm_core::generator::start_date_at_game_open(game_start, 3, 21).0
                } else {
                    league_start
                };
                planned.push((
                    CompetitionDefinition {
                        id: format!("{country_slug}-d{}", tier + 1),
                        name: division_name(&country_label, tier, division_count),
                        r#type: CompetitionType::League,
                        scope: CompetitionScope::Domestic,
                        region_id: Some(region_id.clone()),
                        country_id: Some(country.clone()),
                        required_region_ids: vec![region_id.clone()],
                        priority,
                        format: make_format(CompetitionFormat::LeagueTable),
                        participants: ParticipantSpec {
                            explicit: Some(division_ids.clone()),
                            selector: None,
                        },
                        berths,
                        season_start_month: Some(if country == "BR" && tier > 0 {
                            actual_start.month() as u8
                        } else {
                            league_month
                        }),
                        season_start_day: Some(if country == "BR" {
                            if tier == 0 {
                                28
                            } else {
                                actual_start.day() as u8
                            }
                        } else {
                            1
                        }),
                        name_key: Some(division_tier_name_key(tier, division_count).to_string()),
                        logo: None,
                    },
                    actual_start,
                ));
                priority += 1;
            }
        }

        // National cup
        let cup_month = if ofm_core::nations::is_split_season_country(&country) {
            2
        } else {
            league_month
        };
        let (actual_cup_start, _) =
            ofm_core::generator::start_date_at_game_open(game_start, cup_month, 1);
        let cup_actual_start = actual_cup_start + Duration::days(35);
        planned.push((
            CompetitionDefinition {
                id: format!("{country_slug}-cup"),
                name: format!("{country_label} Cup"),
                r#type: CompetitionType::Cup,
                scope: CompetitionScope::Domestic,
                region_id: Some(region_id.clone()),
                country_id: Some(country.clone()),
                required_region_ids: vec![region_id.clone()],
                priority,
                format: make_format(CompetitionFormat::Knockout),
                participants: ParticipantSpec {
                    explicit: Some(team_ids.clone()),
                    selector: None,
                },
                berths: vec![continental_berth(BerthRule::CupWinner)],
                season_start_month: Some(cup_actual_start.month() as u8),
                season_start_day: Some(cup_actual_start.day() as u8),
                name_key: Some("tournaments.competitions.nationalCup".to_string()),
                logo: None,
            },
            cup_actual_start,
        ));
        priority += 1;

        // Brazil state competitions
        if country == "BR" {
            let labels = [
                (
                    "southeast",
                    "Southeast State Series",
                    "competitionNames.brazilStateSoutheast",
                ),
                (
                    "south",
                    "South State Series",
                    "competitionNames.brazilStateSouth",
                ),
                (
                    "northeast",
                    "Northeast State Series",
                    "competitionNames.brazilStateNortheast",
                ),
                (
                    "north-central-west",
                    "North/Central-West State Series",
                    "competitionNames.brazilStateNorthCentralWest",
                ),
            ];
            let mut pools: BTreeMap<&str, Vec<String>> =
                labels.iter().map(|(id, _, _)| (*id, Vec::new())).collect();
            let mut unknown = Vec::new();
            for team_id in &team_ids {
                let city = game
                    .teams
                    .iter()
                    .find(|team| &team.id == team_id)
                    .map(|team| team.city.as_str())
                    .unwrap_or("");
                if let Some(pool) = brazil_state_region(city) {
                    pools.get_mut(pool).unwrap().push(team_id.clone());
                } else {
                    unknown.push(team_id.clone());
                }
            }
            unknown.sort();
            for team_id in unknown {
                let smallest = labels
                    .iter()
                    .map(|(id, _, _)| *id)
                    .min_by_key(|id| (pools[*id].len(), *id))
                    .unwrap();
                pools.get_mut(smallest).unwrap().push(team_id);
            }
            let state_start = ofm_core::generator::start_date_at_game_open(game_start, 1, 11).0;
            for (id, name, name_key) in labels {
                let participants = pools.remove(id).unwrap_or_default();
                if participants.len() < 2 {
                    continue;
                }
                planned.push((
                    CompetitionDefinition {
                        id: format!("br-state-{id}"),
                        name: name.to_string(),
                        r#type: CompetitionType::Cup,
                        scope: CompetitionScope::Regional,
                        region_id: Some(region_id.clone()),
                        country_id: Some(country.clone()),
                        required_region_ids: vec![region_id.clone()],
                        priority,
                        format: FormatDef {
                            kind: CompetitionFormat::GroupAndKnockout,
                            legs: Some(1),
                            group_size: Some(4),
                            qualifiers_per_group: Some(2),
                            best_third_qualifiers: None,
                        },
                        participants: ParticipantSpec {
                            explicit: Some(participants),
                            selector: None,
                        },
                        berths: Vec::new(),
                        season_start_month: Some(1),
                        season_start_day: Some(11),
                        name_key: Some(name_key.to_string()),
                        logo: None,
                    },
                    state_start,
                ));
                priority += 1;
            }
        }
    }

    // Continental cup
    let continental_team_ids = select_continental_entrants(&game.teams, 2, 16);
    if continental_team_ids.len() >= 4 {
        let mut feeder_regions: Vec<String> = game
            .teams
            .iter()
            .filter(|team| continental_team_ids.contains(&team.id))
            .map(infer_team_region_id)
            .collect();
        feeder_regions.sort();
        feeder_regions.dedup();
        let format_kind = if continental_team_ids.len() >= 8 {
            CompetitionFormat::GroupAndKnockout
        } else {
            CompetitionFormat::Knockout
        };
        let (continental_start, _) =
            ofm_core::generator::start_date_at_game_open(game_start, 10, 1);
        planned.push((
            CompetitionDefinition {
                id: "continental-champions-cup".to_string(),
                name: "Continental Champions Cup".to_string(),
                r#type: CompetitionType::ContinentalClub,
                scope: CompetitionScope::Continental,
                name_key: Some("tournaments.competitions.continentalChampionsCup".to_string()),
                region_id: None,
                country_id: None,
                required_region_ids: feeder_regions,
                priority,
                format: make_format(format_kind),
                participants: ParticipantSpec {
                    explicit: Some(continental_team_ids),
                    selector: None,
                },
                berths: Vec::new(),
                season_start_month: Some(10),
                season_start_day: Some(1),
                logo: None,
            },
            continental_start,
        ));
    }

    planned
}

// ── Foundation competitions ─────────────────────────────────────────

pub fn finalize_brazil_state_competition(competition: &mut League) {
    competition.rules.counts_in_season_flow = false;
    competition.rules.knockout_round_gap_days = 7;
}

pub fn build_foundation_competitions(game: &Game) -> Vec<League> {
    let game_start = game.clock.start_date;
    let season = preseason_league_year(&game.clock);
    build_foundation_competition_plan(game, game_start)
        .iter()
        .filter_map(|(def, start)| {
            let mut competition =
                ofm_core::generator::build_explicit_competition(def, season, *start)?;
            if *start <= game_start {
                ofm_core::catchup::simulate_past_fixtures(
                    &mut competition,
                    &game.players,
                    game_start,
                );
            }
            if competition.id.starts_with("br-state-") {
                finalize_brazil_state_competition(&mut competition);
            }
            Some(competition)
        })
        .collect()
}

// ── International windows ───────────────────────────────────────────

pub fn ensure_international_windows(game: &mut Game) {
    let now = game.clock.current_date;
    let opens_in_world_cup_summer =
        ofm_core::world_cup::is_world_cup_summer(now.year()) && (6..=8).contains(&now.month());
    if opens_in_world_cup_summer
        && ofm_core::world_cup::schedule_world_cup_if_due(game, now + Duration::days(2))
    {
        for national_team in game.national_teams.iter_mut() {
            national_team.fixtures.clear();
        }
        return;
    }

    let window_dates =
        ofm_core::national_team::international_window_dates(preseason_season_start(&game.clock));
    if window_dates.is_empty() {
        return;
    }

    let needs_fixtures = game
        .national_teams
        .iter()
        .all(|team| team.fixtures.is_empty());
    let qualifying_running = game
        .competitions
        .iter()
        .any(ofm_core::world_cup::is_world_cup_qualifying);
    let leads_into_world_cup =
        ofm_core::world_cup::season_leads_into_world_cup(preseason_season_start(&game.clock));
    let starts_qualifying = ofm_core::world_cup::season_starts_world_cup_qualifying(
        preseason_season_start(&game.clock),
    );
    if needs_fixtures && !qualifying_running {
        if starts_qualifying {
            ofm_core::world_cup::schedule_world_cup_qualifying(
                game,
                preseason_season_start(&game.clock).year() + 2,
                &window_dates,
            );
        } else if leads_into_world_cup {
            ofm_core::world_cup::schedule_world_cup_qualifying(
                game,
                preseason_season_start(&game.clock).year() + 1,
                &window_dates,
            );
        } else {
            ofm_core::national_team::schedule_national_team_friendlies(
                &mut game.national_teams,
                &window_dates,
                &mut rand::rng(),
            );
        }
    }

    let reserved_dates = if leads_into_world_cup || starts_qualifying || qualifying_running {
        ofm_core::national_team::international_window_span_dates(&window_dates)
    } else {
        window_dates.clone()
    };
    for competition in &mut game.competitions {
        if ofm_core::world_cup::is_world_cup_competition(competition)
            || ofm_core::world_cup::is_world_cup_qualifying(competition)
        {
            continue;
        }
        ofm_core::schedule::shift_fixtures_off_reserved_dates(competition, &reserved_dates);
    }
    ofm_core::schedule::append_south_american_preseason_friendlies(
        &mut game.competitions,
        &reserved_dates,
    );
    ofm_core::schedule::append_other_preseason_friendlies(&mut game.competitions, &reserved_dates);
}

// ── Multi-competition foundations ────────────────────────────────────

pub fn ensure_multi_competition_foundations(game: &mut Game) {
    if game.national_teams.is_empty() {
        game.national_teams = build_national_teams(game);
    }
    if game.competitions.is_empty() {
        game.competitions = build_foundation_competitions(game);
    }
    if game.active_region_ids.is_empty() {
        game.active_region_ids = game
            .competitions
            .iter()
            .filter_map(|competition| competition.region_id.clone())
            .collect();
        game.active_region_ids.sort();
        game.active_region_ids.dedup();
    }
    if game.active_competition_ids.is_empty() {
        game.active_competition_ids = game
            .competitions
            .iter()
            .map(|competition| competition.id.clone())
            .collect();
    }
    ensure_international_windows(game);
    game.sync_legacy_league();
}

// ── Rebuild competitions for management date ────────────────────────

pub fn rebuild_competitions_for_management_date(
    game: &mut Game,
    management_date: DateTime<Utc>,
) {
    let players = &game.players;
    for competition in &mut game.competitions {
        if ofm_core::world_cup::is_world_cup_competition(competition)
            || ofm_core::world_cup::is_world_cup_qualifying(competition)
        {
            continue;
        }
        let (start, is_mid_season) = ofm_core::generator::start_date_at_game_open(
            management_date,
            competition.season_start_month,
            competition.season_start_day,
        );
        let season = start.year() as u32;
        match competition.rules.format {
            CompetitionFormat::LeagueTable => {
                ofm_core::schedule::regenerate_league_for_season(competition, season, start)
            }
            CompetitionFormat::GroupAndKnockout => {
                ofm_core::group_stage::regenerate_for_season(competition, season, start)
            }
            CompetitionFormat::Knockout => {
                ofm_core::schedule::regenerate_knockout_for_season(competition, season, start)
            }
        }
        if is_mid_season {
            ofm_core::catchup::simulate_past_fixtures(competition, players, management_date);
        }
    }

    let existing: std::collections::HashSet<String> = game
        .competitions
        .iter()
        .map(|competition| competition.id.clone())
        .collect();
    let season = preseason_league_year(&game.clock);
    let mut missing_states: Vec<(League, DateTime<Utc>)> =
        build_foundation_competition_plan(game, management_date)
            .into_iter()
            .filter(|(definition, _)| {
                definition.id.starts_with("br-state-") && !existing.contains(&definition.id)
            })
            .filter_map(|(definition, start)| {
                let mut competition =
                    ofm_core::generator::build_explicit_competition(&definition, season, start)?;
                finalize_brazil_state_competition(&mut competition);
                Some((competition, start))
            })
            .collect();
    for (competition, start) in &mut missing_states {
        if *start <= management_date {
            ofm_core::catchup::simulate_past_fixtures(competition, &game.players, management_date);
        }
    }
    game.competitions
        .extend(missing_states.into_iter().map(|(c, _)| c));
}

// ── Simulation scope ────────────────────────────────────────────────

pub fn resolve_simulation_scope(
    game: &Game,
    team_id: &str,
    requested_region_ids: Option<Vec<String>>,
    requested_competition_ids: Option<Vec<String>>,
) -> Result<(Vec<String>, Vec<String>), String> {
    use std::collections::BTreeSet;

    let managed_team = game
        .teams
        .iter()
        .find(|team| team.id == team_id)
        .ok_or("be.error.teamNotFound".to_string())?;

    let mut active_region_ids: BTreeSet<String> = requested_region_ids
        .unwrap_or_default()
        .into_iter()
        .collect();
    active_region_ids.insert(infer_team_region_id(managed_team));

    let mut active_competition_ids: BTreeSet<String> = requested_competition_ids
        .unwrap_or_default()
        .into_iter()
        .filter(|competition_id| {
            game.competitions
                .iter()
                .any(|competition| competition.id == *competition_id)
        })
        .collect();

    for competition in game.competitions.iter().filter(|competition| {
        competition
            .participant_ids
            .iter()
            .any(|participant_id| participant_id == team_id)
    }) {
        active_competition_ids.insert(competition.id.clone());
    }

    if active_competition_ids.is_empty() {
        for competition in &game.competitions {
            let required_regions = competition_required_region_ids(competition);
            if required_regions.is_empty()
                || required_regions
                    .iter()
                    .all(|region_id| active_region_ids.contains(region_id))
            {
                active_competition_ids.insert(competition.id.clone());
            }
        }
    }

    for competition in game
        .competitions
        .iter()
        .filter(|competition| active_competition_ids.contains(&competition.id))
    {
        for region_id in competition_required_region_ids(competition) {
            active_region_ids.insert(region_id);
        }
    }

    let mut resolved_region_ids: Vec<String> = active_region_ids.into_iter().collect();
    resolved_region_ids.sort();

    let mut resolved_competition_ids: Vec<String> = active_competition_ids.into_iter().collect();
    resolved_competition_ids.sort_by_key(|competition_id| {
        game.competitions
            .iter()
            .find(|competition| competition.id == *competition_id)
            .map(|competition| competition.priority)
            .unwrap_or(u32::MAX)
    });

    Ok((resolved_region_ids, resolved_competition_ids))
}

// ── Team season anchor ──────────────────────────────────────────────

pub fn team_season_anchor(game: &Game, team_id: &str) -> Option<DateTime<Utc>> {
    let team = game.teams.iter().find(|team| team.id == team_id)?;
    let country = if team.football_nation.is_empty() {
        &team.country
    } else {
        &team.football_nation
    };
    if country == "BR" {
        let season_year = game.clock.start_date.year();
        return Utc
            .with_ymd_and_hms(season_year - 1, 12, 15, 0, 0, 0)
            .single();
    }
    let competition = game.competitions.iter().find(|c| {
        c.kind == CompetitionType::League && c.participant_ids.iter().any(|id| id == team_id)
    })?;
    competition
        .fixtures
        .iter()
        .filter(|fixture| fixture.competition != FixtureCompetition::Friendly)
        .filter(|fixture| {
            fixture.home_team_id == team_id || fixture.away_team_id == team_id
        })
        .filter_map(|fixture| chrono::NaiveDate::parse_from_str(&fixture.date, "%Y-%m-%d").ok())
        .min()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|date| {
            DateTime::<Utc>::from_naive_utc_and_offset(date, Utc)
                - Duration::days(PRESEASON_ANCHOR_BUFFER_DAYS)
        })
}

// ── Game clock from world metadata ──────────────────────────────────

pub fn world_start_year(
    startup_options: &StartupOptions,
    metadata: &ofm_core::generator::WorldDataMetadata,
) -> i32 {
    match metadata.kind {
        ofm_core::generator::WorldDataKind::HistoricalSnapshot => {
            metadata.base_year.unwrap_or(startup_options.start_year)
        }
        ofm_core::generator::WorldDataKind::RosterBaseline => startup_options.start_year,
    }
}

pub fn game_clock_for_world(
    startup_options: &StartupOptions,
    metadata: &ofm_core::generator::WorldDataMetadata,
) -> Result<GameClock, String> {
    let start_year = world_start_year(startup_options, metadata);
    let mut clock = GameClock::new(start_date_for_year(start_year)?);
    clock.current_date = match metadata.kind {
        ofm_core::generator::WorldDataKind::HistoricalSnapshot => metadata
            .snapshot_date
            .as_deref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc))
            .unwrap_or(current_date_for_phase(
                start_year,
                startup_options.start_phase,
            )?),
        ofm_core::generator::WorldDataKind::RosterBaseline => {
            current_date_for_phase(startup_options.start_year, startup_options.start_phase)?
        }
    };
    Ok(clock)
}

// ── History ─────────────────────────────────────────────────────────

pub fn apply_generated_past_history(game: &mut Game, startup_options: &StartupOptions) {
    ofm_core::history_generation::generate_past_world_history(
        game,
        startup_options.start_year,
        startup_options.history_depth_years,
    );
}

// ── Build game from world data ──────────────────────────────────────

pub fn build_game_from_world_data(
    clock: GameClock,
    manager: Manager,
    startup_options: &StartupOptions,
    world: ofm_core::generator::WorldData,
) -> (Game, StatsState) {
    let game_start = clock.start_date;
    let defined_competitions: Vec<League> = world
        .competition_definitions
        .as_ref()
        .map(|file| {
            let mut comps = ofm_core::generator::resolve_definitions(
                file,
                &world,
                preseason_league_year(&clock),
                game_start,
            );
            for comp in &mut comps {
                let (_, is_mid_season) = ofm_core::generator::start_date_at_game_open(
                    game_start,
                    comp.season_start_month,
                    comp.season_start_day,
                );
                if is_mid_season {
                    ofm_core::catchup::simulate_past_fixtures(comp, &world.players, game_start);
                }
            }
            comps
        })
        .unwrap_or_default();

    let ofm_core::generator::WorldData {
        teams,
        players,
        staff,
        managers,
        competitions,
        national_teams,
        default_active_regions,
        default_active_competitions,
        league,
        news,
        stats,
        world_history,
        metadata,
        extra_translations,
        ..
    } = world;

    let mut game = Game::new(clock, manager, teams, players, staff, vec![]);
    if game
        .staff
        .iter()
        .any(|staff_member| staff_member.team_id.is_none())
    {
        game.available_staff_market_last_activity_date =
            Some(game.clock.current_date.format("%Y-%m-%d").to_string());
    }
    ofm_core::generator::repair_opening_youth_academies(&mut game);

    let competitions = if defined_competitions.is_empty() {
        competitions
    } else {
        defined_competitions
    };

    match metadata.kind {
        ofm_core::generator::WorldDataKind::HistoricalSnapshot => {
            game.managers.extend(
                managers
                    .into_iter()
                    .filter(|existing_manager| existing_manager.id != game.manager.id),
            );
            game.competitions = competitions;
            game.national_teams = national_teams;
            game.active_region_ids = default_active_regions;
            game.active_competition_ids = default_active_competitions;
            game.league = league;
            game.promote_legacy_league();
            game.news = news;
            game.world_history = world_history;
            game.extra_translations = extra_translations;
            ensure_multi_competition_foundations(&mut game);
            ofm_core::season_context::refresh_game_context(&mut game);
            (game, stats)
        }
        ofm_core::generator::WorldDataKind::RosterBaseline => {
            game.competitions = competitions;
            game.extra_translations = extra_translations;
            ensure_multi_competition_foundations(&mut game);
            apply_generated_past_history(&mut game, startup_options);
            (game, StatsState::default())
        }
    }
}

// ── Bootstrap helpers ───────────────────────────────────────────────

pub fn has_existing_world_context(game: &Game, stats_state: &StatsState) -> bool {
    !game.competitions.is_empty()
        || game.league.is_some()
        || !game.news.is_empty()
        || !stats_state.player_matches.is_empty()
        || !stats_state.team_matches.is_empty()
}

pub fn bootstrap_existing_world_takeover(
    game: &mut Game,
    team_id: &str,
    stats_state: StatsState,
) -> Result<StatsState, String> {
    let team = game
        .teams
        .iter()
        .find(|t| t.id == team_id)
        .ok_or("be.error.teamNotFound".to_string())?;
    let team_name = team.name.clone();

    ofm_core::ai_hiring::seed_ai_managers(game);

    let takeover_date = game.clock.current_date.format("%Y-%m-%d").to_string();
    let incumbent_manager_id = game
        .teams
        .iter()
        .find(|candidate| candidate.id == team_id)
        .and_then(|candidate| candidate.manager_id.clone());

    if incumbent_manager_id.as_deref() != Some(game.manager.id.as_str()) {
        let fired = ofm_core::firing::fire_ai_manager_for_team(game, team_id, &takeover_date);
        if !fired {
            if let Some(team) = game
                .teams
                .iter_mut()
                .find(|candidate| candidate.id == team_id)
            {
                team.manager_id = None;
            }
        }
        ofm_core::job_offers::hire_manager(game, team_id, &takeover_date)?;
    }

    let staff_msg = ofm_core::messages::staff_advice_message(&team_name, team_id, &takeover_date);
    game.messages.push(staff_msg);
    ofm_core::player_events::generate_takeover_contract_review_message(game);
    ofm_core::season_context::refresh_game_context(game);

    Ok(stats_state)
}

pub fn bootstrap_season_start(game: &mut Game, team_id: &str) -> Result<StatsState, String> {
    let team = game
        .teams
        .iter()
        .find(|t| t.id == team_id)
        .ok_or("be.error.teamNotFound".to_string())?;
    let team_name = team.name.clone();

    game.manager.hire(team_id.to_string());
    if let Some(t) = game.teams.iter_mut().find(|t| t.id == team_id) {
        t.manager_id = Some(game.manager.id.clone());
    }
    game.manager_id = game.manager.id.clone();
    ofm_core::ai_hiring::seed_ai_managers(game);

    let season_start = preseason_season_start(&game.clock);
    let team_ids: Vec<String> = game.teams.iter().map(|t| t.id.clone()).collect();
    let league_name = default_league_name();
    let mut league = ofm_core::schedule::generate_league(
        &league_name,
        preseason_league_year(&game.clock),
        &team_ids,
        season_start,
    );
    let friendlies = ofm_core::schedule::generate_preseason_friendlies(&team_ids, season_start, 4);
    ofm_core::schedule::append_fixtures(&mut league, friendlies);
    game.league = Some(league);
    ofm_core::season_context::refresh_game_context(game);

    let date_str = game.clock.current_date.to_rfc3339();
    let welcome_msg = ofm_core::messages::welcome_message(&team_name, team_id, &date_str);
    game.messages.push(welcome_msg);

    let season_msg = ofm_core::messages::season_schedule_message(
        &league_name,
        &season_start.format(&long_date_format()).to_string(),
        &date_str,
    );
    game.messages.push(season_msg);

    let team_names: Vec<String> = game.teams.iter().map(|team| team.name.clone()).collect();
    game.news.push(ofm_core::news::season_preview_article(
        &team_names,
        &date_str,
    ));

    let staff_msg = ofm_core::messages::staff_advice_message(&team_name, team_id, &date_str);
    game.messages.push(staff_msg);

    ofm_core::player_events::generate_takeover_contract_review_message(game);

    Ok(StatsState::default())
}

fn competitive_fixture_count_for_team(game: &Game, team_id: &str) -> usize {
    game.league
        .as_ref()
        .map(|league| {
            league
                .fixtures
                .iter()
                .filter(|fixture| {
                    fixture.counts_for_league_standings()
                        && (fixture.home_team_id == team_id || fixture.away_team_id == team_id)
                })
                .count()
        })
        .unwrap_or_default()
}

fn completed_competitive_fixture_count_for_team(game: &Game, team_id: &str) -> usize {
    game.league
        .as_ref()
        .map(|league| {
            league
                .fixtures
                .iter()
                .filter(|fixture| {
                    fixture.counts_for_league_standings()
                        && fixture.status == domain::league::FixtureStatus::Completed
                        && (fixture.home_team_id == team_id || fixture.away_team_id == team_id)
                })
                .count()
        })
        .unwrap_or_default()
}

pub fn bootstrap_midseason_takeover(
    game: &mut Game,
    team_id: &str,
) -> Result<StatsState, String> {
    let team = game
        .teams
        .iter()
        .find(|t| t.id == team_id)
        .ok_or("be.error.teamNotFound".to_string())?;
    let team_name = team.name.clone();

    ofm_core::ai_hiring::seed_ai_managers(game);

    let season_start = preseason_season_start(&game.clock);
    let league_name = default_league_name();
    let team_ids: Vec<String> = game.teams.iter().map(|t| t.id.clone()).collect();
    game.league = Some(ofm_core::schedule::generate_league(
        &league_name,
        preseason_league_year(&game.clock),
        &team_ids,
        season_start,
    ));
    game.clock.current_date = season_start;
    ofm_core::season_context::refresh_game_context(game);

    let total_fixtures = competitive_fixture_count_for_team(game, team_id);
    let target_completed = (total_fixtures / 2).max(1);
    let mut stats_state = StatsState::default();
    let mut safeguard_days = 0usize;
    while completed_competitive_fixture_count_for_team(game, team_id) < target_completed {
        let mut captures = Vec::new();
        ofm_core::turn::process_day_with_capture(game, &mut |capture| captures.push(capture));
        for capture in captures {
            stats_state.append(capture);
        }
        safeguard_days += 1;
        if safeguard_days > 240 {
            break;
        }
    }

    let takeover_date = game.clock.current_date.format("%Y-%m-%d").to_string();
    let _ = ofm_core::firing::fire_ai_manager_for_team(game, team_id, &takeover_date);
    ofm_core::job_offers::hire_manager(game, team_id, &takeover_date)?;

    let staff_msg = ofm_core::messages::staff_advice_message(&team_name, team_id, &takeover_date);
    game.messages.push(staff_msg);
    ofm_core::player_events::generate_takeover_contract_review_message(game);
    ofm_core::season_context::refresh_game_context(game);

    Ok(stats_state)
}

// ── Bootstrap team selection (the main entry point) ─────────────────

pub fn bootstrap_team_selection(
    game: &mut Game,
    team_id: &str,
    start_phase: StartPhase,
    stats_state: StatsState,
) -> Result<StatsState, String> {
    let stats_state = if has_existing_world_context(game, &stats_state) {
        bootstrap_existing_world_takeover(game, team_id, stats_state)?
    } else {
        match start_phase {
            StartPhase::SeasonStart => bootstrap_season_start(game, team_id)?,
            StartPhase::MidSeason => bootstrap_midseason_takeover(game, team_id)?,
        }
    };

    ofm_core::transfers::seed_opening_ai_loan_market(game);
    Ok(stats_state)
}
