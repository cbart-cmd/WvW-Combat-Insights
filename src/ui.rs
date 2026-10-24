//! Independent combat windows and per-window Nexus options.

use crate::{
    app::{self, Report},
    danger::{self, short_number},
    evtc::{Fight, HealingCoverage, Player, Team},
    i18n::{self, Language, Message, Text},
    settings::{self, DETAILS, HEALING, SUMMARY, Settings, WindowSettings},
};
use nexus::{
    data_link::mumble::{UiState, get_mumble_link, map_type},
    data_link::read_nexus_link,
    imgui::{
        Condition, MenuItem, MouseButton, Selectable, Slider, StyleColor, StyleVar, TextureId, Ui,
        Window,
    },
};
use std::sync::{Arc, LazyLock, Mutex};

const DIM: [f32; 4] = [0.7, 0.7, 0.7, 1.0];
const TITLES: [Text; 3] = [Text::Overview, Text::Highlights, Text::Healing];
const SHORT_TITLES: [Text; 3] = [Text::Overview, Text::HighlightsShort, Text::Healing];

#[cfg(not(test))]
fn texture_id(identifier: String, bytes: &[u8]) -> Option<TextureId> {
    nexus::texture::get_texture_or_create_from_memory(identifier, bytes).map(|texture| texture.id())
}

#[cfg(test)]
fn texture_id(_identifier: String, bytes: &[u8]) -> Option<TextureId> {
    // Headless ImGui records textured geometry without a Nexus renderer or GPU.
    assert!(!bytes.is_empty());
    Some(TextureId::new(1))
}

struct View {
    report: Option<Arc<Report>>,
    team: Option<Team>,
    select_team: bool,
    sort: usize,
    ascending: bool,
    needs_position: bool,
    focus: bool,
    last_size: [f32; 2],
    highlights: Option<HighlightCache>,
}

struct HighlightCache {
    report: Arc<Report>,
    team: Team,
    sort: usize,
    ascending: bool,
    rows: Vec<usize>,
    values: Vec<[String; 4]>,
}

impl HighlightCache {
    fn new(report: &Arc<Report>, team: Team, sort: usize, ascending: bool) -> Self {
        let fight = &report.fight;
        let rows = danger::highlight_rows(fight, team, sort, ascending);
        let values = rows
            .iter()
            .map(|&index| {
                let p = &fight.players[index];
                [
                    short_number(fight.dps(p)),
                    short_number(p.damage as f64),
                    p.strips.to_string(),
                    short_number(p.down_contribution as f64),
                ]
            })
            .collect();
        Self {
            report: Arc::clone(report),
            team,
            sort,
            ascending,
            rows,
            values,
        }
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            report: None,
            team: None,
            select_team: true,
            sort: 4,
            ascending: false,
            needs_position: true,
            focus: false,
            last_size: [0.0; 2],
            highlights: None,
        }
    }
}

impl View {
    fn set_report(&mut self, report: Option<Arc<Report>>) {
        if self.report.as_ref().map(|r| &r.path) != report.as_ref().map(|r| &r.path) {
            self.team = None;
            self.select_team = true;
        }
        self.report = report;
    }
}

struct Extra {
    id: u64,
    kind: usize,
    view: View,
    settings: WindowSettings,
}

#[derive(Default)]
struct Workspace {
    views: [View; 3],
    extras: Vec<Extra>,
    next_id: u64,
    dirty: bool,
    initialized: bool,
    shortcut_hidden: bool,
}

impl Workspace {
    fn select_combat(&mut self, settings: &mut Settings, report: Arc<Report>) {
        for (view, w) in self.views.iter_mut().zip(&mut settings.windows) {
            view.set_report(Some(Arc::clone(&report)));
            w.follow_latest = false;
        }
        for extra in &mut self.extras {
            extra.view.set_report(Some(Arc::clone(&report)));
            extra.settings.follow_latest = false;
        }
    }

    fn follow_latest(&mut self, settings: &mut Settings, follow: bool) {
        for w in &mut settings.windows {
            w.follow_latest = follow;
        }
        for extra in &mut self.extras {
            extra.settings.follow_latest = follow;
        }
    }
}

static WORKSPACE: LazyLock<Mutex<Workspace>> = LazyLock::new(|| Mutex::new(Workspace::default()));

enum Action {
    ShowTeam(Arc<Report>, Team),
    Duplicate(usize, Arc<Report>, Option<Team>),
    SelectReport(Arc<Report>),
    FollowLatest(bool),
    ShowWeapons(bool),
}

fn is_wvw_map(map_id: u32, kind: u32) -> bool {
    map_id != 0
        && matches!(
            kind,
            map_type::WVW_ETERNAL_BATTLEGROUNDS
                | map_type::WVW_BLUE_BORDERLANDS
                | map_type::WVW_GREEN_BORDERLANDS
                | map_type::WVW_RED_BORDERLANDS
                | map_type::WVW_REWARD
                | map_type::WVW_OBSIDIAN_SANCTUM
                | map_type::WVW_EDGE_OF_THE_MISTS
                | map_type::WVW_LOUNGE
        )
}

pub(crate) fn in_wvw() -> bool {
    get_mumble_link().is_some_and(|link| is_wvw_map(link.read_map_id(), link.read_map_type()))
}

pub(crate) fn in_combat() -> bool {
    get_mumble_link().is_some_and(|link| link.read_ui_state().contains(UiState::IS_IN_COMBAT))
}

pub fn toggle() {
    let mut st = app::state();
    let show = !st.settings.windows.iter().any(|w| w.enabled && w.visible);
    for w in &mut st.settings.windows {
        if w.enabled {
            w.visible = show;
        }
    }
    WORKSPACE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .shortcut_hidden = !show;
    st.settings.save();
}

fn select_report(view: &mut View, history: &[Arc<Report>], follow_latest: bool) {
    if follow_latest || view.report.is_none() {
        view.set_report(history.first().cloned());
    } else if let Some(selected) = &view.report
        && let Some(updated) = history
            .iter()
            .find(|report| report.path == selected.path || report.replaces(selected))
    {
        view.set_report(Some(Arc::clone(updated)));
    }
}

fn select_display(
    view: &mut View,
    history: &[Arc<Report>],
    live: Option<&Arc<Report>>,
    follow: bool,
) {
    if follow && let Some(live) = live {
        view.set_report(Some(Arc::clone(live)));
    } else {
        select_report(view, history, follow);
    }
}

pub fn render(ui: &Ui) {
    if !should_render(read_nexus_link().map(|link| link.is_gameplay), in_wvw()) {
        return;
    }
    let mut st = app::state();
    let mut workspace = WORKSPACE.lock().unwrap_or_else(|e| e.into_inner());
    if !workspace.initialized {
        workspace.views[HEALING].sort = st.settings.healing_sort;
        let follow = st.settings.windows[SUMMARY].follow_latest;
        workspace.follow_latest(&mut st.settings, follow);
        workspace.initialized = true;
    }
    let before = st.settings.clone();
    let language = i18n::current();
    let combat = crate::live::in_player_combat();
    let mut actions = Vec::new();
    for kind in 0..3 {
        let config = st.settings.clone();
        let mut w = config.windows[kind].clone();
        if !w.enabled || !w.visible || (combat && w.hide_in_combat) {
            continue;
        }
        select_display(
            &mut workspace.views[kind],
            &st.history,
            st.live.as_ref(),
            w.follow_latest,
        );
        render_window(
            ui,
            kind,
            &format!("FAD_Window_{kind}"),
            &mut workspace.views[kind],
            &mut w,
            &config,
            &st.history,
            &st.status,
            language,
            &mut actions,
            false,
        );
        st.settings.windows[kind] = w;
    }
    let shortcut_hidden = workspace.shortcut_hidden;
    for extra in &mut workspace.extras {
        if shortcut_hidden {
            continue;
        }
        if !st.settings.windows[extra.kind].enabled || (combat && extra.settings.hide_in_combat) {
            continue;
        }
        select_display(
            &mut extra.view,
            &st.history,
            st.live.as_ref(),
            extra.settings.follow_latest,
        );
        render_window(
            ui,
            extra.kind,
            &format!("FAD_Extra_{}", extra.id),
            &mut extra.view,
            &mut extra.settings,
            &st.settings,
            &st.history,
            &st.status,
            language,
            &mut actions,
            true,
        );
    }
    workspace.extras.retain(|e| e.settings.visible);
    for action in actions {
        match action {
            Action::SelectReport(report) => {
                workspace.select_combat(&mut st.settings, report);
            }
            Action::FollowLatest(follow) => {
                workspace.follow_latest(&mut st.settings, follow);
            }
            Action::ShowWeapons(show) => {
                st.settings.show_weapons = show;
            }
            Action::ShowTeam(report, team) => {
                let w = &mut st.settings.windows[DETAILS];
                w.enabled = true;
                w.visible = true;
                let view = &mut workspace.views[DETAILS];
                view.report = Some(report);
                view.team = Some(team);
                view.select_team = true;
                view.focus = true;
            }
            Action::Duplicate(kind, report, team) => {
                workspace.next_id += 1;
                let id = workspace.next_id;
                let mut w = st.settings.windows[kind].clone();
                w.visible = true;
                w.fixed_position = false;
                let offset = 24.0 * ((id - 1) % 8 + 1) as f32;
                w.position[0] += offset;
                w.position[1] += offset;
                workspace.extras.push(Extra {
                    id,
                    kind,
                    settings: w,
                    view: View {
                        report: Some(report),
                        team,
                        focus: true,
                        sort: if kind == HEALING {
                            st.settings.healing_sort
                        } else {
                            4
                        },
                        ..Default::default()
                    },
                });
            }
        }
    }
    workspace.dirty |= st.settings != before;
    if workspace.dirty && !ui.is_mouse_down(MouseButton::Left) {
        st.settings.save();
        workspace.dirty = false;
    }
}

fn should_render(gameplay: Option<bool>, wvw: bool) -> bool {
    gameplay == Some(true) && wvw
}

fn visible_position(position: [f32; 2], width: f32, display: [f32; 2]) -> [f32; 2] {
    [
        position[0].clamp(0.0, (display[0] - width.min(display[0])).max(0.0)),
        position[1].clamp(0.0, (display[1] - 80.0).max(0.0)),
    ]
}

#[allow(clippy::too_many_arguments)]
fn render_window(
    ui: &Ui,
    kind: usize,
    id: &str,
    view: &mut View,
    w: &mut WindowSettings,
    config: &Settings,
    history: &[Arc<Report>],
    status: &Message,
    language: Language,
    actions: &mut Vec<Action>,
    extra: bool,
) {
    let display = ui.io().display_size;
    let width = w.width.min(display[0].max(1.0));
    let position = visible_position(w.position, width, display);
    let title = language.text(SHORT_TITLES[kind]);
    let _padding = ui.push_style_var(StyleVar::WindowPadding([4.0, 3.0]));
    let _spacing = ui.push_style_var(StyleVar::ItemSpacing([3.0, 1.0]));
    let _frame = ui.push_style_var(StyleVar::FramePadding([2.0, 0.0]));
    let background = [0.06, 0.06, 0.07, w.opacity];
    let _title = ui.push_style_color(StyleColor::TitleBg, background);
    let _active = ui.push_style_color(StyleColor::TitleBgActive, background);
    let _bg = ui.push_style_color(StyleColor::WindowBg, background);
    let mut open = w.visible;
    let mut window = Window::new(format!("{title}###{id}"))
        .opened(&mut open)
        .resizable(true)
        .collapsible(false)
        .collapsed(false, Condition::Always)
        .bg_alpha(w.opacity)
        .movable(!w.locked && !w.fixed_position)
        .size(
            [width, w.height],
            if view.last_size != [w.width, w.height] {
                Condition::Always
            } else {
                Condition::Never
            },
        )
        .size_constraints(
            [125.0_f32.min(display[0]), 60.0],
            [1000.0_f32.min(display[0]), (display[1] - 30.0).max(100.0)],
        )
        .position(
            position,
            if w.fixed_position || view.needs_position {
                Condition::Always
            } else {
                Condition::Never
            },
        )
        .focused(view.focus);
    // Keep a recovery path when a resolution change puts a free window outside the viewport.
    if !w.fixed_position && position != w.position {
        window = window.position(position, Condition::Always);
    }
    window.build(ui, || {
        view.needs_position = false;
        view.focus = false;
        let size = ui.window_size();
        view.last_size = size;
        w.width = size[0];
        w.height = size[1];
        if !w.fixed_position {
            w.position = ui.window_pos();
        }
        if ui.is_window_hovered() && ui.is_mouse_clicked(MouseButton::Right) {
            ui.open_popup("window_menu");
        }
        if let Some(_popup) = ui.begin_popup("window_menu") {
            window_menu(ui, kind, view, w, config, history, language, actions, extra);
        }
        if !status.is_empty()
            && !status.is_history_loaded()
            && !extra
            && !view
                .report
                .as_ref()
                .is_some_and(|r| crate::live::is_report(r))
        {
            ui.text_wrapped(status.render(language));
        }
        let Some(report) = view.report.clone() else {
            if status.is_empty() || status.is_history_loaded() {
                ui.text_wrapped(language.text(Text::Waiting));
            }
            return;
        };
        if let Some(live) = &report.live {
            ui.text_colored(
                DIM,
                language.text(if live.limited {
                    Text::LiveLimit
                } else if live.finished {
                    Text::LiveFinished
                } else {
                    Text::Live
                }),
            );
            if ui.is_item_hovered() {
                ui.tooltip_text(language.text(Text::LiveTip));
            }
            if kind != HEALING && report.fight.players.iter().all(|p| p.team.is_none()) {
                ui.text_wrapped(language.text(Text::LiveNoTeams));
            }
        }
        match kind {
            SUMMARY => overview(ui, &report, w.opacity, language, actions),
            DETAILS => highlights(ui, &report, view, config, w.columns, language),
            HEALING => healing(ui, &report, view, w.columns, language),
            _ => unreachable!(),
        }
        if ui.is_window_hovered()
            && !ui.is_any_item_hovered()
            && let Some(report) = &view.report
        {
            ui.tooltip_text(format!(
                "{}\n{}\n{}\n{}",
                log_label(report, language),
                language.text(Text::ContextTip),
                language.format(
                    Text::ParseTime,
                    &[
                        format!("{:.2}", report.parse_time.as_secs_f32()),
                        report.path.display().to_string()
                    ]
                ),
                if kind == HEALING {
                    language.text(Text::HealingScope)
                } else {
                    ""
                }
            ));
        }
    });
    w.visible = open;
}

fn log_label(report: &Report, language: Language) -> String {
    if let Some(live) = &report.live {
        return format!(
            "{} · {} · {}",
            language.text(if live.finished {
                Text::LiveFinished
            } else {
                Text::Live
            }),
            report.path.display(),
            i18n::format_duration(report.fight.duration_ms, language)
        );
    }
    format!(
        "{} · {}",
        i18n::format_log_name(&report.file_name, language),
        i18n::format_duration(report.fight.duration_ms, language)
    )
}

#[allow(clippy::too_many_arguments)]
fn window_menu(
    ui: &Ui,
    kind: usize,
    view: &mut View,
    w: &mut WindowSettings,
    config: &Settings,
    history: &[Arc<Report>],
    language: Language,
    actions: &mut Vec<Action>,
    _extra: bool,
) {
    if let Some(report) = &view.report {
        ui.text_disabled(log_label(report, language));
    }
    if let Some(_menu) = ui.begin_menu(language.widget(Text::HistoryMenu)) {
        if MenuItem::new(language.widget(Text::FollowLatest))
            .build_with_ref(ui, &mut w.follow_latest)
        {
            actions.push(Action::FollowLatest(w.follow_latest));
        }
        ui.separator();
        for (index, report) in history.iter().enumerate() {
            let selected = view.report.as_ref().is_some_and(|r| r.path == report.path);
            if Selectable::new(format!("{}##log{index}", log_label(report, language)))
                .selected(selected)
                .close_popups(false)
                .build(ui)
            {
                actions.push(Action::SelectReport(Arc::clone(report)));
            }
            if ui.is_item_hovered() {
                ui.tooltip_text(report.path.display().to_string());
            }
        }
        ui.separator();
        if MenuItem::new(language.widget(Text::Latest)).build(ui) {
            app::analyze_latest();
        }
        if MenuItem::new(language.format(Text::Reprocess, &[config.history_size.to_string()]))
            .build(ui)
        {
            app::reprocess_history();
        }
    }
    if kind == DETAILS
        && let Some(_menu) = ui.begin_menu(language.widget(Text::Team))
    {
        if let Some(report) = &view.report {
            let current = view
                .team
                .unwrap_or_else(|| initial_team(&report.fight, config.initial_team));
            let teams = if report.live.is_some() {
                display_teams(report)
            } else {
                participating_teams(&report.fight)
            };
            for team in teams {
                let _color = ui.push_style_color(StyleColor::Text, team.rgba());
                if MenuItem::new(language.team(team))
                    .selected(team == current)
                    .build(ui)
                {
                    view.team = Some(team);
                    view.select_team = true;
                }
            }
        }
    }
    if kind == DETAILS {
        let mut show = config.show_weapons;
        if MenuItem::new(language.widget(Text::ShowWeapons)).build_with_ref(ui, &mut show) {
            actions.push(Action::ShowWeapons(show));
        }
    }
    if kind != SUMMARY
        && let Some(_menu) = ui.begin_menu(language.widget(Text::StatsMenu))
    {
        let labels = metric_labels(kind);
        column_options(ui, kind, &mut w.columns, language);
        ui.separator();
        ui.text_disabled(language.text(Text::SortBy));
        for (metric, label) in labels.iter().copied().enumerate() {
            if MenuItem::new(language.widget(label))
                .selected(view.sort == metric)
                .build(ui)
            {
                view.sort = metric;
            }
        }
        if kind == DETAILS
            && MenuItem::new(language.widget(Text::HighlightScore))
                .selected(view.sort == 4)
                .build(ui)
        {
            view.sort = 4;
            view.ascending = false;
        }
        ui.separator();
        MenuItem::new(language.widget(Text::InverseSort)).build_with_ref(ui, &mut view.ascending);
    }
    if let Some(_menu) = ui.begin_menu(language.widget(Text::StyleMenu)) {
        ui.set_next_item_width(140.0);
        Slider::new(language.widget(Text::Width), 125.0, 1000.0).build(ui, &mut w.width);
        let mut opacity = (w.opacity * 100.0).round() as i32;
        ui.set_next_item_width(140.0);
        if Slider::new(language.widget(Text::Opacity), 0, 100).build(ui, &mut opacity) {
            w.opacity = opacity as f32 / 100.0;
        }
        if MenuItem::new(language.widget(Text::ResetStyle)).build(ui) {
            let defaults = WindowSettings::for_kind(kind);
            w.width = defaults.width;
            w.height = defaults.height;
            w.opacity = defaults.opacity;
        }
    }
    if let Some(_menu) = ui.begin_menu(language.widget(Text::PositionMenu)) {
        MenuItem::new(language.widget(Text::LockWindow)).build_with_ref(ui, &mut w.locked);
        MenuItem::new(language.widget(Text::FixedPosition))
            .build_with_ref(ui, &mut w.fixed_position);
        ui.set_next_item_width(140.0);
        if ui.input_float2("X / Y", &mut w.position).build() {
            w.normalize(kind);
            view.needs_position = true;
        }
        if MenuItem::new(language.widget(Text::ResetPosition)).build(ui) {
            w.position = WindowSettings::for_kind(kind).position;
            view.needs_position = true;
        }
    }
    if let Some(report) = &view.report {
        ui.separator();
        if kind != SUMMARY && MenuItem::new(language.widget(Text::Copy)).build(ui) {
            copy_report(ui, kind, report, view, config, language, false);
        }
        if kind == DETAILS && MenuItem::new(language.widget(Text::Chat)).build(ui) {
            copy_report(ui, kind, report, view, config, language, true);
        }
        if kind != SUMMARY && MenuItem::new(language.widget(Text::OpenAnother)).build(ui) {
            actions.push(Action::Duplicate(kind, Arc::clone(report), view.team));
        }
    }
}

fn metric_labels(kind: usize) -> &'static [Text] {
    if kind == HEALING {
        &[
            Text::Total,
            Text::HealingAmount,
            Text::Barrier,
            Text::Cleanses,
        ]
    } else {
        &[
            Text::Dps,
            Text::Damage,
            Text::Strips,
            Text::DownContribution,
        ]
    }
}

fn team_order(fight: &Fight) -> Vec<Team> {
    if fight.pov_team == Some(Team::Own) {
        return vec![Team::Own];
    }
    let mut teams = Team::ALL.to_vec();
    teams.sort_by_key(|team| Some(*team) != fight.pov_team);
    teams
}

fn display_teams(report: &Report) -> Vec<Team> {
    if report.live.is_some() {
        report.fight.pov_team.into_iter().collect()
    } else {
        team_order(&report.fight)
    }
}

fn participating_teams(fight: &Fight) -> Vec<Team> {
    team_order(fight)
        .into_iter()
        .filter(|team| fight.players.iter().any(|p| p.team == Some(*team)))
        .collect()
}

fn button_color(team: Team) -> [f32; 4] {
    let color = team.rgba();
    [color[0] * 0.42, color[1] * 0.42, color[2] * 0.42, 1.0]
}

fn team_damage(fight: &Fight, team: Team) -> u64 {
    fight
        .players
        .iter()
        .filter(|p| p.team == Some(team))
        .fold(0u64, |total, p| total.saturating_add(p.damage))
}

fn team_strips(fight: &Fight, team: Team) -> u64 {
    fight
        .players
        .iter()
        .filter(|p| p.team == Some(team))
        .map(|p| u64::from(p.strips))
        .sum()
}

fn team_down_contribution(fight: &Fight, team: Team) -> u64 {
    fight
        .players
        .iter()
        .filter(|p| p.team == Some(team))
        .fold(0u64, |total, p| total.saturating_add(p.down_contribution))
}

fn overview(
    ui: &Ui,
    report: &Arc<Report>,
    opacity: f32,
    language: Language,
    actions: &mut Vec<Action>,
) {
    let fight = &report.fight;
    let width = ui.content_region_avail()[0];
    let kills_width = ui.calc_text_size(language.text(Text::Kills))[0].max(30.0);
    let totals: Vec<_> = display_teams(report)
        .into_iter()
        .map(|team| {
            (
                team,
                short_number(team_damage(fight, team) as f64),
                team_strips(fight, team).to_string(),
                short_number(team_down_contribution(fight, team) as f64),
            )
        })
        .collect();
    let damage_width = totals.iter().fold(
        ui.calc_text_size(language.text(Text::Damage))[0],
        |width, (_, damage, _, _)| width.max(ui.calc_text_size(damage)[0]),
    ) + 5.0;
    let strips_width = totals.iter().fold(
        ui.calc_text_size(language.text(Text::Strips))[0],
        |width, (_, _, strips, _)| width.max(ui.calc_text_size(strips)[0]),
    ) + 5.0;
    let contribution_width = totals.iter().fold(
        ui.calc_text_size(language.text(Text::DownContributionShort))[0],
        |width, (_, _, _, contribution)| width.max(ui.calc_text_size(contribution)[0]),
    ) + 5.0;
    let scale = (width * 0.80
        / (kills_width + 4.0 + damage_width + strips_width + contribution_width))
        .min(1.0);
    let columns = vec![
        MetricColumn {
            metric: 1,
            width: (kills_width + 4.0) * scale,
        },
        MetricColumn {
            metric: 2,
            width: damage_width * scale,
        },
        MetricColumn {
            metric: 3,
            width: strips_width * scale,
        },
        MetricColumn {
            metric: 4,
            width: contribution_width * scale,
        },
    ];
    let heading = ui.cursor_screen_pos();
    let height = ui.text_line_height();
    ui.invisible_button("summary_heading", [width, height]);
    if ui.is_item_hovered()
        && ui.io().mouse_pos[0] >= heading[0] + width - contribution_width * scale
    {
        ui.tooltip_text("Down contribution");
    }
    ui.get_window_draw_list().add_text(
        heading,
        DIM,
        clipped_name(
            ui,
            language.text(Text::PlayersShort),
            (width - columns.iter().map(|c| c.width).sum::<f32>() - 3.0).max(0.0),
        ),
    );
    draw_metrics(
        ui,
        heading,
        width,
        &columns,
        &[
            String::new(),
            language.text(Text::Kills).into(),
            language.text(Text::Damage).into(),
            language.text(Text::Strips).into(),
            language.text(Text::DownContributionShort).into(),
        ],
        DIM,
    );
    for (team, damage, strips, contribution) in &totals {
        let team = *team;
        let count = fight
            .players
            .iter()
            .filter(|p| p.team == Some(team))
            .count();
        let events = crate::evtc::team_event_counts(fight, team);
        let own = Some(team) == fight.pov_team;
        let pos = ui.cursor_screen_pos();
        let height = ui.text_line_height();
        if ui.invisible_button(format!("team_{team:?}"), [width, height]) {
            actions.push(Action::ShowTeam(Arc::clone(report), team));
        }
        let hovered = ui.is_item_hovered();
        let list = ui.get_window_draw_list();
        let mut color = button_color(team);
        color[3] = opacity;
        if hovered {
            for component in &mut color[..3] {
                *component += 0.1;
            }
        }
        let cell_width = (width - columns.iter().map(|c| c.width).sum::<f32>() - 3.0).max(1.0);
        list.add_rect(pos, [pos[0] + cell_width, pos[1] + height], color)
            .filled(true)
            .build();
        let label = clipped_name(ui, &count.to_string(), cell_width);
        let text_width = ui.calc_text_size(&label)[0];
        list.add_text(
            [pos[0] + (cell_width - text_width) / 2.0, pos[1]],
            [1.0; 4],
            &label,
        );
        drop(list);
        draw_metrics(
            ui,
            pos,
            width,
            &columns,
            &[
                String::new(),
                events.kills.to_string(),
                damage.clone(),
                strips.clone(),
                contribution.clone(),
            ],
            [1.0; 4],
        );
        if hovered {
            let name = if own {
                format!(
                    "{} · {}",
                    language.team(team),
                    language.text(Text::OwnTeamShort)
                )
            } else {
                language.team(team).into()
            };
            ui.tooltip_text(format!(
                "{}\n{}: {}\n{}: {}\n{}: {}",
                language.format(
                    Text::TeamEventsTip,
                    &[
                        name,
                        count.to_string(),
                        events.kills.to_string(),
                        events.deaths.to_string(),
                        events.downs.to_string(),
                    ],
                ),
                language.text(Text::Damage),
                team_damage(fight, team),
                language.text(Text::Strips),
                strips,
                language.text(Text::DownContribution),
                team_down_contribution(fight, team)
            ));
        }
    }
}

fn initial_team(fight: &Fight, choice: usize) -> Team {
    match choice {
        1 => fight.pov_team.unwrap_or(Team::Red),
        2..=4 => Team::ALL[choice - 2],
        _ => Team::ALL
            .into_iter()
            .find(|team| Some(*team) != fight.pov_team)
            .unwrap_or(Team::Red),
    }
}

fn column_options(ui: &Ui, kind: usize, mask: &mut u8, language: Language) {
    let _id = ui.push_id("columns");
    ui.text_disabled(language.text(Text::Columns));
    for (metric, label) in metric_labels(kind).iter().copied().enumerate() {
        let bit = 1 << metric;
        let mut enabled = *mask & bit != 0;
        if ui.checkbox(language.widget(label), &mut enabled) {
            let next = if enabled { *mask | bit } else { *mask & !bit };
            if next != 0 {
                *mask = next;
            }
        }
    }
}

struct MetricColumn {
    metric: usize,
    width: f32,
}

fn metric_heading_label(label: Text) -> Text {
    if matches!(label, Text::DownContribution) {
        Text::DownContributionShort
    } else {
        label
    }
}

fn metric_columns<const N: usize>(
    ui: &Ui,
    kind: usize,
    mask: u8,
    values: &[[String; N]],
    language: Language,
) -> Vec<MetricColumn> {
    let mut columns: Vec<_> = metric_labels(kind)
        .iter()
        .copied()
        .enumerate()
        .filter(|(metric, _)| mask & (1 << metric) != 0)
        .map(|(metric, label)| MetricColumn {
            metric,
            width: values.iter().fold(
                ui.calc_text_size(language.text(metric_heading_label(label)))[0],
                |width, row| width.max(ui.calc_text_size(&row[metric])[0]),
            ) + 5.0,
        })
        .collect();
    let total: f32 = columns.iter().map(|c| c.width).sum();
    let scale = (ui.content_region_avail()[0] * 0.68 / total.max(1.0)).min(1.0);
    for column in &mut columns {
        column.width *= scale;
    }
    columns
}

fn draw_metrics(
    ui: &Ui,
    pos: [f32; 2],
    width: f32,
    columns: &[MetricColumn],
    values: &[String],
    color: [f32; 4],
) {
    let list = ui.get_window_draw_list();
    let mut right = pos[0] + width;
    for column in columns.iter().rev() {
        let text = clipped_name(ui, &values[column.metric], (column.width - 3.0).max(0.0));
        list.add_text([right - ui.calc_text_size(&text)[0], pos[1]], color, text);
        right -= column.width;
    }
}

fn metric_heading(
    ui: &Ui,
    left: &str,
    kind: usize,
    columns: &[MetricColumn],
    view: &mut View,
    language: Language,
) -> Option<usize> {
    let pos = ui.cursor_screen_pos();
    let width = ui.content_region_avail()[0];
    let total: f32 = columns.iter().map(|c| c.width).sum();
    let clicked = ui.invisible_button("heading", [width, ui.text_line_height()]);
    ui.get_window_draw_list().add_text(
        pos,
        DIM,
        clipped_name(ui, left, (width - total - 3.0).max(0.0)),
    );
    draw_metrics(
        ui,
        pos,
        width,
        columns,
        &metric_labels(kind)
            .iter()
            .map(|&label| language.text(metric_heading_label(label)).into())
            .collect::<Vec<_>>(),
        DIM,
    );
    let mut right = pos[0] + width;
    let mut selected = None;
    for column in columns.iter().rev() {
        if ui.io().mouse_pos[0] >= right - column.width && ui.io().mouse_pos[0] <= right {
            selected = Some(column.metric);
        }
        right -= column.width;
    }
    if clicked {
        if let Some(metric) = selected.filter(|metric| *metric != view.sort) {
            view.sort = metric;
            view.ascending = false;
        } else {
            view.ascending = !view.ascending;
        }
    }
    selected.filter(|_| ui.is_item_hovered())
}

fn highlights(
    ui: &Ui,
    report: &Arc<Report>,
    view: &mut View,
    config: &Settings,
    mask: u8,
    language: Language,
) {
    let fight = &report.fight;
    let teams = if report.live.is_some() {
        display_teams(report)
    } else {
        participating_teams(fight)
    };
    if !view.team.is_some_and(|team| teams.contains(&team)) {
        let preferred = initial_team(fight, config.initial_team);
        view.team = teams
            .iter()
            .copied()
            .find(|team| *team == preferred)
            .or_else(|| teams.first().copied());
        view.select_team = true;
    }
    let Some(mut team) = view.team else {
        ui.text_wrapped(language.text(Text::NoActivity));
        return;
    };
    let requested = view.select_team.then_some(team);
    view.select_team = false;
    if let Some(_tabs) = ui.tab_bar("teams") {
        for available in teams {
            let _color = ui.push_style_color(StyleColor::Text, available.rgba());
            if let Some(_tab) = nexus::imgui::TabItem::new(format!(
                "{}###team_{available:?}",
                language.team(available)
            ))
            .flags(if Some(available) == requested {
                nexus::imgui::TabItemFlags::SET_SELECTED
            } else {
                nexus::imgui::TabItemFlags::empty()
            })
            .begin(ui)
            {
                if requested.is_none() || requested == Some(available) {
                    view.team = Some(available);
                }
            }
        }
    }
    team = view.team.unwrap_or(team);
    if !view.highlights.as_ref().is_some_and(|cache| {
        Arc::ptr_eq(&cache.report, report)
            && cache.team == team
            && cache.sort == view.sort
            && cache.ascending == view.ascending
    }) {
        view.highlights = Some(HighlightCache::new(report, team, view.sort, view.ascending));
    }
    let columns = metric_columns(
        ui,
        DETAILS,
        mask,
        &view.highlights.as_ref().unwrap().values,
        language,
    );
    let hovered_metric = metric_heading(ui, language.team(team), DETAILS, &columns, view, language);
    let cache = view.highlights.as_ref().unwrap();
    if cache.rows.is_empty() {
        ui.text_wrapped(language.text(Text::NoActivity));
        return;
    }
    if hovered_metric == Some(3) {
        ui.tooltip_text("Down contribution");
    } else if ui.is_item_hovered() {
        ui.tooltip_text(format!(
            "{}\n{}",
            language.format(
                Text::Active,
                &[
                    language.team(team).into(),
                    fight
                        .players
                        .iter()
                        .filter(|p| p.team == Some(team) && (p.damage > 0 || p.strips > 0))
                        .count()
                        .to_string()
                ]
            ),
            language.text(Text::ContextTip)
        ));
    }
    let spacing = ui.clone_style().item_spacing[1];
    for (position, &index) in cache.rows.iter().enumerate() {
        let p = &fight.players[index];
        let height = ui.text_line_height();
        let step = height + spacing;
        let weapons = config.show_weapons && !p.weapons.is_empty();
        let lines = if weapons { 2.0 } else { 1.0 };
        let width = ui.content_region_avail()[0];
        let row_height = lines * step - spacing;
        let pos = ui.cursor_screen_pos();
        if !ui.is_rect_visible(pos, [pos[0] + width, pos[1] + row_height]) {
            ui.dummy([width, row_height]);
            continue;
        }
        let tooltip = format!(
            "{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}",
            language.text(Text::Dps),
            short_number(fight.dps(p)),
            language.text(Text::Damage),
            p.damage,
            language.text(Text::Strips),
            p.strips,
            language.text(Text::DownContribution),
            p.down_contribution,
            language.format(
                Text::DamageTip,
                &[p.strike_damage.to_string(), p.condi_damage.to_string()]
            )
        );
        player_row(
            ui,
            p,
            index,
            false,
            &cache.values[position],
            &columns,
            &tooltip,
            language,
        );
        if weapons {
            let _id = ui.push_id(index as i32);
            weapon_row(ui, p, language);
        }
    }
}

fn clipped_name(ui: &Ui, name: &str, width: f32) -> String {
    if ui.calc_text_size(name)[0] <= width {
        return name.into();
    }
    if ui.calc_text_size("...")[0] > width {
        return String::new();
    }
    let mut text = String::new();
    for ch in name.chars() {
        let candidate = format!("{text}{ch}...");
        if ui.calc_text_size(&candidate)[0] > width {
            break;
        }
        text.push(ch);
    }
    text.push_str("...");
    text
}

fn player_row(
    ui: &Ui,
    player: &Player,
    index: usize,
    profession_only: bool,
    values: &[String],
    columns: &[MetricColumn],
    extra: &str,
    language: Language,
) {
    let pos = ui.cursor_screen_pos();
    let width = ui.content_region_avail()[0];
    let height = ui.text_line_height();
    let value_width: f32 = columns.iter().map(|c| c.width).sum();
    ui.invisible_button(format!("player{index}"), [width, height]);
    let hovered = ui.is_item_hovered();
    let name_left = height + 3.0;
    let name_width = (width - name_left - value_width - 5.0).max(0.0);
    let elite = if profession_only { 0 } else { player.elite };
    let bytes = if profession_only {
        crate::evtc::profession_icon(player.profession)
    } else {
        player.specialization_icon()
    };
    if let Some(bytes) = bytes
        && let Some(texture) = texture_id(
            format!("FIGHT_ANALYSIS_DANGER_SPEC_{}_{}", player.profession, elite),
            bytes,
        )
    {
        ui.get_window_draw_list()
            .add_image(texture, pos, [pos[0] + height, pos[1] + height])
            .build();
    }
    let name = if player.account.is_empty() {
        language.rank_name(&player.name)
    } else {
        &player.name
    };
    let list = ui.get_window_draw_list();
    if hovered {
        list.add_rect(
            pos,
            [pos[0] + width, pos[1] + height],
            [1.0, 1.0, 1.0, 0.08],
        )
        .filled(true)
        .build();
    }
    list.add_text(
        [pos[0] + name_left, pos[1]],
        player.class_color(),
        clipped_name(ui, name, name_width),
    );
    drop(list);
    draw_metrics(ui, pos, width, columns, values, [1.0; 4]);
    if hovered {
        ui.tooltip_text(format!(
            "{}\n{}\n{}\n{}: {}\n{}",
            name,
            player.account,
            language.spec(player.profession, player.elite),
            language.text(Text::Subgroup),
            player
                .subgroup
                .map_or_else(|| "?".into(), |s| s.to_string()),
            extra
        ));
    }
}

fn weapon_row(ui: &Ui, player: &Player, language: Language) {
    let pos = ui.cursor_screen_pos();
    let width = ui.content_region_avail()[0];
    let height = ui.text_line_height();
    ui.invisible_button("weapons", [width, height]);
    let list = ui.get_window_draw_list();
    let mut x = pos[0];
    let mut tooltip = Vec::new();
    for (i, weapon) in player.weapons.iter().enumerate() {
        let underwater = player.underwater_weapons.contains(weapon);
        let name = language.weapon(weapon);
        tooltip.push(if underwater {
            format!("{name} ({})", language.text(Text::UnderwaterWeapon))
        } else {
            name.to_string()
        });
        let label = format!("{}{name}", if i == 0 { "" } else { " · " });
        let color = if underwater {
            [0.3, 0.65, 1.0, 1.0]
        } else {
            DIM
        };
        let remaining = (pos[0] + width - x).max(0.0);
        let text = clipped_name(ui, &label, remaining);
        list.with_clip_rect_intersect(pos, [pos[0] + width, pos[1] + height], || {
            list.add_text([x, pos[1]], color, &text);
        });
        x += ui.calc_text_size(&label)[0];
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(format!(
            "{}\n{}",
            tooltip.join(" · "),
            language.text(Text::WeaponTip)
        ));
    }
}

fn healing(ui: &Ui, report: &Report, view: &mut View, mask: u8, language: Language) {
    let fight = &report.fight;
    match fight.healing_coverage {
        HealingCoverage::Missing | HealingCoverage::Unsupported => {
            ui.text_wrapped(language.text(Text::HealingMissing));
            if !fight
                .players
                .iter()
                .any(|p| p.squad_member && p.cleanses > 0)
            {
                return;
            }
        }
        HealingCoverage::UnknownGroup => {
            ui.text_wrapped(language.text(Text::HealingNoGroup));
            return;
        }
        HealingCoverage::Partial | HealingCoverage::Complete => {}
    }
    let rows = healing_rows(fight, view.sort, view.ascending);
    if rows.is_empty() {
        ui.text_wrapped(language.text(Text::HealingEmpty));
        return;
    }
    let partial = fight.healing_coverage == HealingCoverage::Partial;
    let values: Vec<[String; 4]> = rows
        .iter()
        .map(|index| {
            std::array::from_fn(|metric| healing_text(&fight.players[*index], metric, true))
        })
        .collect();
    let columns = metric_columns(ui, HEALING, mask, &values, language);
    metric_heading(
        ui,
        if partial {
            language.text(Text::PartialShort)
        } else {
            language.text(Text::Player)
        },
        HEALING,
        &columns,
        view,
        language,
    );
    if ui.is_item_hovered() {
        ui.tooltip_text(format!(
            "{}\n{}\n{}",
            language.text(Text::HealingScope),
            if partial {
                language.text(Text::HealingPartial)
            } else {
                ""
            },
            language.text(Text::ContextTip)
        ));
    }
    for index in rows {
        let p = &fight.players[index];
        let tooltip = format!(
            "{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}\n{}",
            language.text(Text::HealingAmount),
            healing_text(p, 1, false),
            language.text(Text::Barrier),
            healing_text(p, 2, false),
            language.text(Text::Total),
            healing_text(p, 0, false),
            language.text(Text::Cleanses),
            healing_text(p, 3, false),
            language.text(Text::PerSecond),
            healing_value(p, view.sort)
                .filter(|_| fight.duration_ms > 0)
                .map_or_else(
                    || "-".into(),
                    |n| short_number(n as f64 / fight.duration_secs())
                ),
            language.text(Text::HealingRecorded),
            if partial {
                language.text(Text::HealingPartial)
            } else {
                ""
            }
        );
        let values: [String; 4] = std::array::from_fn(|metric| healing_text(p, metric, true));
        player_row(ui, p, index, true, &values, &columns, &tooltip, language);
    }
}

fn healing_text(player: &Player, metric: usize, compact: bool) -> String {
    healing_value(player, metric).map_or_else(
        || "-".into(),
        |value| {
            let incomplete = metric == 0
                && (healing_value(player, 1).is_none() || healing_value(player, 2).is_none());
            format!(
                "{}{}",
                if incomplete && !compact { ">=" } else { "" },
                if compact {
                    short_number(value as f64)
                } else {
                    value.to_string()
                }
            )
        },
    )
}

fn copy_report(
    ui: &Ui,
    kind: usize,
    report: &Report,
    view: &View,
    config: &Settings,
    language: Language,
    chat: bool,
) {
    let fight = &report.fight;
    if kind == DETAILS {
        let team = view
            .team
            .unwrap_or_else(|| initial_team(fight, config.initial_team));
        let rows = danger::highlight_rows(fight, team, view.sort, view.ascending);
        let entries: Vec<_> = rows
            .iter()
            .enumerate()
            .map(|(position, &index)| {
                let p = &fight.players[index];
                format!(
                    "{}. {}: {} {}, {} {}, {} {}, {} {}",
                    position + 1,
                    p.name,
                    short_number(fight.dps(p)),
                    language.text(Text::Dps),
                    if chat {
                        short_number(p.damage as f64)
                    } else {
                        p.damage.to_string()
                    },
                    language.text(Text::Damage),
                    p.strips,
                    language.text(Text::Strips),
                    if chat {
                        short_number(p.down_contribution as f64)
                    } else {
                        p.down_contribution.to_string()
                    },
                    language.text(Text::DownContribution)
                )
            })
            .collect();
        ui.set_clipboard_text(format!(
            "{}\n{}",
            if report.live.is_some() {
                format!("{} · {}", language.text(Text::Live), language.team(team))
            } else {
                language.team(team).to_owned()
            },
            entries.join(if chat { " | " } else { "\n" })
        ));
    } else if kind == HEALING {
        let mut text = format!(
            "{} · {}\n{}\n",
            language.text(Text::Healing),
            log_label(report, language),
            if fight.healing_coverage == HealingCoverage::Partial {
                language.text(Text::HealingPartial)
            } else {
                language.text(Text::HealingScope)
            }
        );
        for index in healing_rows(fight, view.sort, view.ascending) {
            let p = &fight.players[index];
            text.push_str(&format!(
                "{}: {} {}, {} {}, {} {}, {} {}\n",
                p.name,
                healing_text(p, 1, false),
                language.text(Text::HealingAmount),
                healing_text(p, 2, false),
                language.text(Text::Barrier),
                healing_text(p, 0, false),
                language.text(Text::Total),
                healing_text(p, 3, false),
                language.text(Text::Cleanses)
            ));
        }
        ui.set_clipboard_text(text);
    }
}

fn metric_available(coverage: HealingCoverage) -> bool {
    matches!(
        coverage,
        HealingCoverage::Partial | HealingCoverage::Complete
    )
}

fn healing_value(player: &Player, metric: usize) -> Option<u64> {
    match metric {
        1 => metric_available(player.healing_coverage).then_some(player.healing),
        2 => metric_available(player.barrier_coverage).then_some(player.barrier),
        3 => player.squad_member.then_some(u64::from(player.cleanses)),
        _ => {
            let healing = healing_value(player, 1);
            let barrier = healing_value(player, 2);
            (healing.is_some() || barrier.is_some())
                .then_some(healing.unwrap_or(0).saturating_add(barrier.unwrap_or(0)))
        }
    }
}

fn healing_rows(fight: &Fight, sort: usize, ascending: bool) -> Vec<usize> {
    let mut rows: Vec<_> = (0..fight.players.len())
        .filter(|i| {
            let p = &fight.players[*i];
            fight.is_squad_member(p) && (healing_value(p, 0).is_some() || p.cleanses > 0)
        })
        .collect();
    rows.sort_by(|a, b| {
        match (
            healing_value(&fight.players[*a], sort),
            healing_value(&fight.players[*b], sort),
        ) {
            (Some(a_value), Some(b_value)) => {
                let order = if ascending {
                    a_value.cmp(&b_value)
                } else {
                    b_value.cmp(&a_value)
                };
                order.then(a.cmp(b))
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.cmp(b),
        }
    });
    rows
}

pub fn options(ui: &Ui) {
    let language = i18n::current();
    let mut st = app::state();
    let before = st.settings.clone();
    let mut workspace = WORKSPACE.lock().unwrap_or_else(|e| e.into_inner());
    let mut follow_latest = None;
    ui.text(language.text(Text::Description));
    ui.text_wrapped(language.text(Text::WvwOnly));
    if let Some(_tabs) = ui.tab_bar("FAD_Options") {
        for (kind, title) in TITLES.into_iter().enumerate() {
            if let Some(_tab) = ui.tab_item(language.widget(title)) {
                let w = &mut st.settings.windows[kind];
                if ui.checkbox(language.widget(Text::Enabled), &mut w.enabled) {
                    w.visible = w.enabled;
                    workspace.views[kind].focus = w.enabled;
                }
                if ui.checkbox(language.widget(Text::Visible), &mut w.visible) && w.visible {
                    w.enabled = true;
                    workspace.views[kind].focus = true;
                }
                ui.checkbox(language.widget(Text::AutoShow), &mut w.auto_show);
                ui.checkbox(language.widget(Text::HideInCombat), &mut w.hide_in_combat);
                ui.separator();
                ui.checkbox(language.widget(Text::FixedPosition), &mut w.fixed_position);
                if ui.input_float2("X / Y", &mut w.position).build() {
                    workspace.views[kind].needs_position = true;
                }
                ui.checkbox(language.widget(Text::LockWindow), &mut w.locked);
                Slider::new(language.widget(Text::Width), 125.0, 1000.0).build(ui, &mut w.width);
                Slider::new(language.widget(Text::Height), 60.0, 1000.0).build(ui, &mut w.height);
                let mut opacity = (w.opacity * 100.0).round() as i32;
                if Slider::new(language.widget(Text::Opacity), 0, 100).build(ui, &mut opacity) {
                    w.opacity = opacity as f32 / 100.0;
                }
                if ui.button(language.widget(Text::ResetStyle)) {
                    let defaults = WindowSettings::for_kind(kind);
                    w.width = defaults.width;
                    w.height = defaults.height;
                    w.opacity = defaults.opacity;
                }
                if ui.button(language.widget(Text::ResetPosition)) {
                    w.position = WindowSettings::for_kind(kind).position;
                    workspace.views[kind].needs_position = true;
                }
                if ui.checkbox(language.widget(Text::FollowLatest), &mut w.follow_latest) {
                    follow_latest = Some(w.follow_latest);
                }
                if kind != SUMMARY {
                    column_options(ui, kind, &mut w.columns, language);
                }
                w.normalize(kind);
                ui.separator();
                match kind {
                    DETAILS => {
                        ui.text_wrapped(language.text(Text::HighlightScore));
                        ui.checkbox(
                            language.widget(Text::ShowWeapons),
                            &mut st.settings.show_weapons,
                        );
                        let choices = [
                            Text::FirstEnemy,
                            Text::OwnTeamShort,
                            Text::Red,
                            Text::Blue,
                            Text::Green,
                        ];
                        if let Some(_combo) = ui.begin_combo(
                            language.widget(Text::InitialTeam),
                            language.text(choices[st.settings.initial_team]),
                        ) {
                            for (i, choice) in choices.into_iter().enumerate() {
                                if Selectable::new(language.widget(choice))
                                    .selected(st.settings.initial_team == i)
                                    .build(ui)
                                {
                                    st.settings.initial_team = i;
                                    workspace.views[DETAILS].team = None;
                                    workspace.views[DETAILS].select_team = true;
                                }
                            }
                        }
                        ui.text_wrapped(language.text(Text::WeaponTip));
                    }
                    HEALING => {
                        let sorts = metric_labels(HEALING);
                        if let Some(_combo) = ui.begin_combo(
                            language.widget(Text::SortBy),
                            language.text(sorts[st.settings.healing_sort]),
                        ) {
                            for (i, sort) in sorts.iter().copied().enumerate() {
                                if Selectable::new(language.widget(sort))
                                    .selected(st.settings.healing_sort == i)
                                    .build(ui)
                                {
                                    st.settings.healing_sort = i;
                                    workspace.views[HEALING].sort = i;
                                    workspace.views[HEALING].ascending = false;
                                }
                            }
                        }
                        ui.text_wrapped(language.text(Text::HealingScope));
                    }
                    _ => {}
                }
            }
        }
    }
    if let Some(follow) = follow_latest {
        workspace.follow_latest(&mut st.settings, follow);
    }
    ui.separator();
    if ui.collapsing_header(
        language.widget(Text::GeneralOptions),
        nexus::imgui::TreeNodeFlags::empty(),
    ) {
        ui.checkbox(
            language.widget(Text::PlayersOnly),
            &mut st.settings.vs_players_only,
        );
        let mut history = st.settings.history_size as i32;
        Slider::new(
            language.widget(Text::HistorySize),
            1,
            settings::MAX_HISTORY_SIZE as i32,
        )
        .build(ui, &mut history);
        st.settings.history_size = history as usize;
        let mut min = st.settings.min_duration_secs as i32;
        Slider::new(language.widget(Text::MinimumDuration), 0, 180).build(ui, &mut min);
        st.settings.min_duration_secs = min as u32;
        ui.input_text(language.widget(Text::LogDir), &mut st.settings.log_dir)
            .build();
        let dir = st
            .log_dir
            .as_ref()
            .map(|d| d.display().to_string())
            .unwrap_or_else(|| language.text(Text::Undetermined).into());
        ui.text_wrapped(language.format(Text::CurrentDir, &[dir]));
        if ui.button(language.widget(Text::Latest)) {
            app::analyze_latest();
        }
        if ui.button(language.format(Text::Reprocess, &[st.settings.history_size.to_string()])) {
            app::reprocess_history();
        }
    }
    ui.text_wrapped(language.text(Text::Shortcut));
    ui.text_wrapped(language.text(Text::HiddenNames));
    if st.settings != before {
        let history_size = st.settings.history_size;
        st.history.truncate(history_size);
        st.history_index = st.history_index.min(st.history.len().saturating_sub(1));
        st.settings.save();
        if st.settings.vs_players_only != before.vs_players_only {
            drop(workspace);
            drop(st);
            app::reanalyze();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn loading_screens_hide_every_window_even_when_mumble_keeps_the_wvw_map() {
        assert!(!super::should_render(Some(false), true));
        assert!(!super::should_render(None, true));
        assert!(!super::should_render(Some(true), false));
        assert!(super::should_render(Some(true), true));
    }

    #[test]
    fn live_shows_only_own_team_while_logs_keep_all_colors() {
        let mut report = Report {
            path: Default::default(),
            file_name: String::new(),
            parse_time: Default::default(),
            live: None,
            fight: Fight {
                pov_team: Some(Team::Green),
                ..Default::default()
            },
        };
        assert_eq!(
            display_teams(&report),
            vec![Team::Green, Team::Red, Team::Blue]
        );
        report.live = Some(app::LiveStatus {
            limited: false,
            finished: false,
        });
        assert_eq!(display_teams(&report), vec![Team::Green]);
        report.fight.pov_team = Some(Team::Own);
        assert_eq!(display_teams(&report), vec![Team::Own]);
        report.fight.pov_team = None;
        assert!(display_teams(&report).is_empty());
    }

    use super::*;

    #[test]
    fn complete_highlights_use_weighted_damage_and_invalidate_cached_reports() {
        let mut report = Arc::new(Report {
            path: "large.zevtc".into(),
            file_name: "large.zevtc".into(),
            fight: Fight {
                players: (0..180)
                    .map(|i| Player {
                        team: Some(Team::ALL[i / 60]),
                        damage: i as u64,
                        strips: if i == 0 { 1 } else { 0 },
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            },
            parse_time: Default::default(),
            live: None,
        });
        assert_eq!(danger::highlight_score(&report.fight.players[0]), 2000);
        let cache = HighlightCache::new(&report, Team::Red, 4, false);
        assert_eq!(cache.rows.len(), 60);
        assert_eq!(&cache.rows[..3], &[0, 59, 58]);
        assert_eq!(cache.values[0][2], "1");
        assert_eq!(
            danger::highlight_rows(&report.fight, Team::Red, 4, true)[0],
            1
        );
        assert_eq!(
            danger::highlight_rows(&report.fight, Team::Red, 1, false)[0],
            59
        );
        Arc::make_mut(&mut report).fight.players[1].damage = u64::MAX;
        assert!(!Arc::ptr_eq(&cache.report, &report));
        assert_eq!(HighlightCache::new(&report, Team::Red, 4, false).rows[0], 1);
        let huge = Player {
            damage: u64::MAX,
            strips: u32::MAX,
            ..Default::default()
        };
        assert!(danger::highlight_score(&huge) > u128::from(u64::MAX));
        let zero = Fight {
            players: vec![Player {
                team: Some(Team::Red),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(danger::highlight_rows(&zero, Team::Red, 4, false).is_empty());
        let weighted = Fight {
            players: vec![
                Player {
                    team: Some(Team::Red),
                    damage: 10_000,
                    strips: 1,
                    ..Default::default()
                },
                Player {
                    team: Some(Team::Red),
                    damage: 9_000,
                    down_contribution: 4_000,
                    ..Default::default()
                },
                Player {
                    team: Some(Team::Red),
                    damage: 9_000,
                    down_contribution: 4_000,
                    ..Default::default()
                },
                Player {
                    team: Some(Team::Red),
                    strips: 2,
                    ..Default::default()
                },
                Player {
                    team: Some(Team::Red),
                    down_contribution: 50_000,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert_eq!(danger::highlight_score(&weighted.players[1]), 13_000);
        assert_eq!(
            danger::highlight_rows(&weighted, Team::Red, 4, false),
            vec![1, 2, 0, 3]
        );
        assert_eq!(
            danger::highlight_rows(&weighted, Team::Red, 1, false),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            danger::highlight_rows(&weighted, Team::Red, 3, true),
            vec![0, 3, 1, 2]
        );
    }

    #[test]
    fn cleanse_only_squad_members_can_rank_without_inventing_healing() {
        let fight = Fight {
            players: vec![
                Player {
                    squad_member: true,
                    cleanses: 12,
                    ..Default::default()
                },
                Player {
                    squad_member: true,
                    cleanses: 5,
                    ..Default::default()
                },
                Player {
                    cleanses: 100,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert_eq!(healing_rows(&fight, 3, false), vec![0, 1]);
        assert_eq!(healing_rows(&fight, 3, true), vec![1, 0]);
        assert_eq!(healing_text(&fight.players[0], 3, false), "12");
        assert_eq!(healing_text(&fight.players[0], 1, false), "-");
        assert_eq!(healing_text(&fight.players[0], 0, false), "-");
        assert_eq!(healing_value(&fight.players[2], 3), None);
    }

    #[test]
    fn own_team_is_first_and_other_colors_remain_stable() {
        for own in Team::ALL {
            let fight = Fight {
                pov_team: Some(own),
                ..Default::default()
            };
            let order = team_order(&fight);
            assert_eq!(order[0], own);
            assert_eq!(order.len(), 3);
            assert_eq!(initial_team(&fight, 1), own);
            assert_ne!(initial_team(&fight, 0), own);
        }
    }

    #[test]
    fn choosing_a_different_fight_resets_team_but_refreshing_it_preserves_manual_color() {
        let report = |path: &str, own| {
            Arc::new(Report {
                path: path.into(),
                file_name: path.into(),
                fight: Fight {
                    pov_team: Some(own),
                    event_range: Some((1000, 2000)),
                    ..Default::default()
                },
                parse_time: Default::default(),
                live: None,
            })
        };
        let mut view = View::default();
        view.set_report(Some(report("one", Team::Blue)));
        view.team = Some(Team::Red);
        view.set_report(Some(report("one", Team::Blue)));
        assert_eq!(view.team, Some(Team::Red));
        view.set_report(Some(report("two", Team::Green)));
        assert_eq!(view.team, None);
        assert!(view.select_team);
        assert_eq!(
            initial_team(
                &view.report.as_ref().unwrap().fight,
                Settings::default().initial_team
            ),
            Team::Green
        );

        let mut provisional = (*report("live-1", Team::Green)).clone();
        provisional.live = Some(app::LiveStatus {
            limited: false,
            finished: true,
        });
        view.set_report(Some(Arc::new(provisional)));
        let real = report("real", Team::Green);
        select_report(&mut view, &[Arc::clone(&real)], false);
        assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &real));
    }

    #[test]
    fn summary_down_contribution_sums_only_the_selected_team() {
        let mut fight = Fight {
            players: vec![
                Player {
                    team: Some(Team::Red),
                    down_contribution: 1200,
                    ..Default::default()
                },
                Player {
                    team: Some(Team::Red),
                    down_contribution: 3400,
                    ..Default::default()
                },
                Player {
                    team: Some(Team::Green),
                    down_contribution: 9000,
                    ..Default::default()
                },
                Player {
                    down_contribution: 50_000,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert_eq!(team_down_contribution(&fight, Team::Red), 4600);
        assert_eq!(team_down_contribution(&fight, Team::Green), 9000);
        assert_eq!(team_down_contribution(&fight, Team::Blue), 0);
        fight.players[0].down_contribution = u64::MAX;
        assert_eq!(team_down_contribution(&fight, Team::Red), u64::MAX);
    }

    #[test]
    fn pinned_reports_survive_new_logs_and_history_eviction() {
        let report = |name: &str| {
            Arc::new(Report {
                path: name.into(),
                file_name: name.into(),
                fight: Fight::default(),
                parse_time: Default::default(),
                live: None,
            })
        };
        let old = report("old");
        let new = report("new");
        let mut view = View {
            report: Some(Arc::clone(&old)),
            ..Default::default()
        };
        select_report(&mut view, &[Arc::clone(&new)], false);
        assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &old));
        select_report(&mut view, &[], false);
        assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &old));
        select_report(&mut view, &[Arc::clone(&new)], true);
        assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &new));
        let refreshed = report("new");
        select_report(&mut view, &[Arc::clone(&refreshed)], false);
        assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &refreshed));
        assert!(!Arc::ptr_eq(view.report.as_ref().unwrap(), &new));
        let mut live = report("");
        Arc::make_mut(&mut live).live = Some(app::LiveStatus {
            limited: false,
            finished: false,
        });
        select_display(&mut view, &[Arc::clone(&new)], Some(&live), false);
        assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &new));
        select_display(&mut view, &[Arc::clone(&new)], Some(&live), true);
        assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &live));
        let frozen = Arc::clone(&live);
        Arc::make_mut(&mut live).fight.duration_ms = 10_000;
        select_display(&mut view, &[Arc::clone(&new)], Some(&live), false);
        assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &frozen));
        select_display(&mut view, &[Arc::clone(&new)], Some(&live), true);
        assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &live));
        Arc::make_mut(&mut live).live.as_mut().unwrap().finished = true;
        select_display(&mut view, &[Arc::clone(&live)], None, true);
        assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &live));
        select_display(&mut view, &[Arc::clone(&new)], None, true);
        assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &new));
        let mut empty = View::default();
        select_display(&mut empty, &[Arc::clone(&new)], Some(&live), false);
        assert!(Arc::ptr_eq(empty.report.as_ref().unwrap(), &new));
    }

    #[test]
    fn positions_stay_recoverable_after_resolution_changes() {
        assert_eq!(
            visible_position([3000.0, 2000.0], 380.0, [1920.0, 1080.0]),
            [1540.0, 1000.0]
        );
        assert_eq!(
            visible_position([100.0, 100.0], 380.0, [200.0, 50.0]),
            [0.0, 0.0]
        );
    }

    #[test]
    fn healing_has_no_row_cap_and_sorts_only_available_squad_metrics() {
        let mut fight = Fight {
            pov_team: Some(Team::Green),
            pov_subgroup: Some(1),
            healing_coverage: HealingCoverage::Partial,
            players: (0..12)
                .map(|i| Player {
                    team: Some(Team::Green),
                    subgroup: Some(1),
                    squad_member: true,
                    healing: (i + 1) * 100,
                    barrier: (12 - i) * 10,
                    healing_coverage: HealingCoverage::Partial,
                    barrier_coverage: HealingCoverage::Partial,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        assert_eq!(
            healing_rows(&fight, 0, false),
            (0..12).rev().collect::<Vec<_>>()
        );
        assert_eq!(healing_rows(&fight, 1, true), (0..12).collect::<Vec<_>>());
        assert_eq!(healing_rows(&fight, 2, false), (0..12).collect::<Vec<_>>());
        fight.players[11].squad_member = false;
        assert_eq!(
            healing_rows(&fight, 0, false),
            (0..11).rev().collect::<Vec<_>>()
        );
        fight.players[0].barrier_coverage = HealingCoverage::Missing;
        assert_eq!(healing_value(&fight.players[0], 2), None);
        assert_eq!(healing_value(&fight.players[0], 0), Some(100));
        assert_eq!(healing_text(&fight.players[0], 0, true), "100");
        assert_eq!(healing_text(&fight.players[0], 0, false), ">=100");
        assert_eq!(healing_text(&fight.players[0], 2, true), "-");
        assert_eq!(healing_rows(&fight, 2, true).last(), Some(&0));
        fight.players[0].healing_coverage = HealingCoverage::Missing;
        assert_eq!(healing_value(&fight.players[0], 0), None);
        assert_eq!(healing_rows(&fight, 0, false).len(), 10);
        fight.players.clear();
        assert!(healing_rows(&fight, 0, false).is_empty());
    }

    #[test]
    fn participating_colors_and_team_damage_use_recorded_rosters() {
        let fight = Fight {
            pov_team: Some(Team::Green),
            players: vec![
                Player {
                    team: Some(Team::Red),
                    damage: 100,
                    strips: 4,
                    ..Default::default()
                },
                Player {
                    team: Some(Team::Red),
                    damage: 250,
                    strips: 7,
                    ..Default::default()
                },
                Player {
                    team: Some(Team::Green),
                    damage: 50,
                    ..Default::default()
                },
                Player {
                    team: None,
                    damage: 999,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert_eq!(participating_teams(&fight), vec![Team::Green, Team::Red]);
        assert_eq!(team_damage(&fight, Team::Red), 350);
        assert_eq!(team_damage(&fight, Team::Green), 50);
        assert_eq!(team_damage(&fight, Team::Blue), 0);
        assert_eq!(team_strips(&fight, Team::Red), 11);
        assert_eq!(team_strips(&fight, Team::Blue), 0);
        assert!(participating_teams(&Fight::default()).is_empty());
    }

    #[test]
    fn selecting_history_synchronizes_all_windows_including_hidden_and_extra() {
        let report = Arc::new(Report {
            path: "selected.zevtc".into(),
            file_name: "selected.zevtc".into(),
            fight: Fight::default(),
            parse_time: Default::default(),
            live: None,
        });
        let mut workspace = Workspace::default();
        let mut settings = Settings::default();
        workspace.extras.push(Extra {
            id: 1,
            kind: DETAILS,
            view: View::default(),
            settings: WindowSettings::for_kind(DETAILS),
        });
        workspace.select_combat(&mut settings, Arc::clone(&report));
        for view in &workspace.views {
            assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &report));
        }
        assert!(Arc::ptr_eq(
            workspace.extras[0].view.report.as_ref().unwrap(),
            &report
        ));
        assert!(settings.windows.iter().all(|w| !w.follow_latest));
        assert!(!workspace.extras[0].settings.follow_latest);
        workspace.follow_latest(&mut settings, true);
        assert!(settings.windows.iter().all(|w| w.follow_latest));
        assert!(workspace.extras[0].settings.follow_latest);
    }

    #[test]
    fn accepts_only_wvw_maps_with_valid_map_ids() {
        for kind in 0..=32 {
            assert_eq!(
                is_wvw_map(38, kind),
                [9, 10, 11, 12, 13, 14, 15, 18].contains(&kind)
            );
            assert!(!is_wvw_map(0, kind));
        }
        assert!(!is_wvw_map(38, u32::MAX));
    }

    #[test]
    fn color_cells_have_high_white_text_contrast() {
        for team in Team::ALL {
            let c = button_color(team);
            let linear = |v: f32| {
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            let luminance = 0.2126 * linear(c[0]) + 0.7152 * linear(c[1]) + 0.0722 * linear(c[2]);
            assert!(1.05 / (luminance + 0.05) >= 4.5);
        }
    }

    #[test]
    fn renders_all_three_windows_in_every_language_without_a_game() {
        let mut context = nexus::imgui::Context::create();
        context.set_ini_filename(None);
        context.io_mut().display_size = [1920.0, 1080.0];
        context.io_mut().delta_time = 1.0 / 60.0;
        context.fonts().build_rgba32_texture();
        let mut report = Arc::new(Report {
            path: "20261019-221400.zevtc".into(),
            file_name: "20261019-221400.zevtc".into(),
            fight: Fight {
                duration_ms: 92_000,
                pov_team: Some(Team::Green),
                pov_subgroup: Some(1),
                healing_coverage: HealingCoverage::Partial,
                players: vec![
                    Player {
                        name: "A long player name for clipping".into(),
                        team: Some(Team::Red),
                        damage: 1_690_000,
                        down_contribution: 120_000,
                        strips: 32,
                        weapons: vec!["Spear", "Greatsword", "Sword", "Focus"],
                        underwater_weapons: vec!["Spear"],
                        ..Default::default()
                    },
                    Player {
                        name: "Healer".into(),
                        profession: 1,
                        team: Some(Team::Green),
                        damage: 10_000,
                        weapons: vec!["Spear"],
                        underwater_weapons: vec!["Spear"],
                        subgroup: Some(1),
                        squad_member: true,
                        healing: 820_000,
                        barrier: 240_000,
                        healing_coverage: HealingCoverage::Partial,
                        barrier_coverage: HealingCoverage::Partial,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            parse_time: Default::default(),
            live: None,
        });
        Arc::make_mut(&mut report)
            .fight
            .players
            .extend((0..10).map(|i| Player {
                name: format!("Healer {i}"),
                team: Some(Team::Green),
                subgroup: Some(1),
                squad_member: true,
                healing: 500_000 + i * 1000,
                barrier: 120_000,
                healing_coverage: HealingCoverage::Partial,
                barrier_coverage: HealingCoverage::Partial,
                ..Default::default()
            }));
        let config = Settings::default();
        for font_scale in [1.0, 1.5] {
            context.io_mut().font_global_scale = font_scale;
            for language in Language::ALL {
                for kind in 0..3 {
                    for mask in 1..=15 {
                        if kind == SUMMARY && mask != 6 {
                            continue;
                        }
                        let mut view = View {
                            report: Some(Arc::clone(&report)),
                            ..Default::default()
                        };
                        let mut w = WindowSettings::for_kind(kind);
                        w.width = 175.0;
                        w.opacity = 0.75;
                        w.height = 0.0;
                        w.columns = mask;
                        w.visible = true;
                        let mut actions = vec![];
                        let mut vertices = 0;
                        let mut height = 0.0;
                        let mut text_height = 0.0;
                        let mut background_alpha = false;
                        let mut aquatic_tint = false;
                        for frame in 0..3 {
                            let ui = context.frame();
                            render_window(
                                &ui,
                                kind,
                                "headless",
                                &mut view,
                                &mut w,
                                &config,
                                &[Arc::clone(&report)],
                                &Message::default(),
                                language,
                                &mut actions,
                                false,
                            );
                            Window::new("measure###headless").build(&ui, || {
                                assert!(
                                    (ui.window_size()[0] - w.width).abs() <= 1.0,
                                    "{language:?} {kind}: {:?}",
                                    ui.window_size()
                                );
                                assert!(!ui.is_window_collapsed());
                                height = ui.window_size()[1];
                                text_height = ui.text_line_height();
                                assert!(
                                    ui.calc_text_size(clipped_name(&ui, "A very long name", 37.0))
                                        [0]
                                        <= 37.0
                                );
                                assert!(clipped_name(&ui, "Name", 0.0).is_empty());
                            });
                            let draw_data = ui.render();
                            vertices = draw_data.total_vtx_count;
                            if frame == 2 {
                                background_alpha = draw_data
                                    .draw_lists()
                                    .flat_map(|list| list.vtx_buffer())
                                    .any(|v| v.col[3] == 191 && v.col[0] < 30 && v.col[1] < 30);
                                aquatic_tint = draw_data
                                    .draw_lists()
                                    .flat_map(|list| list.vtx_buffer())
                                    .any(|v| {
                                        (75..=78).contains(&v.col[0])
                                            && (164..=167).contains(&v.col[1])
                                            && v.col[2] == 255
                                            && v.col[3] == 255
                                    });
                            }
                        }
                        assert!(vertices > 0, "{language:?} {kind}");
                        assert!(
                            background_alpha,
                            "75% opacity must reach the rendered background"
                        );
                        if kind == DETAILS {
                            assert!(aquatic_tint, "Verified aquatic weapons must be tinted blue");
                        }
                        let max_lines = match kind {
                            SUMMARY => 6.0,
                            HEALING => 14.0,
                            _ => 5.0,
                        };
                        assert!(
                            height <= max_lines * text_height + 12.0,
                            "{language:?} {kind}: height {height}"
                        );
                        assert!(w.visible);
                        assert!(actions.is_empty());
                    }
                }
            }
        }
        context.io_mut().font_global_scale = 1.0;
        {
            let ui = context.frame();
            Window::new("row-metrics")
                .size([175.0, 500.0], Condition::Always)
                .build(&ui, || {
                    for kind in [DETAILS, HEALING] {
                        for mask in 1..=15 {
                            let values = [[
                                "999.9k".into(),
                                "999.9m".into(),
                                "123456".into(),
                                "999".into(),
                            ]];
                            let columns =
                                metric_columns(&ui, kind, mask, &values, Language::English);
                            assert_eq!(columns.len(), (mask as u8).count_ones() as usize);
                            assert!(
                                columns.iter().map(|c| c.width).sum::<f32>()
                                    <= ui.content_region_avail()[0] * 0.68 + 0.01
                            );
                            for column in columns {
                                let text = clipped_name(
                                    &ui,
                                    &values[0][column.metric],
                                    (column.width - 3.0).max(0.0),
                                );
                                assert!(ui.calc_text_size(text)[0] <= column.width);
                            }
                        }
                    }
                    overview(&ui, &report, 0.75, Language::English, &mut vec![]);
                    assert_eq!(
                        ui.item_rect_size()[1],
                        ui.text_line_height(),
                        "Summary cells must be text-height"
                    );
                    let mut view = View::default();
                    healing(&ui, &report, &mut view, 6, Language::English);
                    assert_eq!(
                        ui.item_rect_size()[1],
                        ui.text_line_height(),
                        "Healing must occupy one text line per player"
                    );
                    highlights(&ui, &report, &mut view, &config, 6, Language::English);
                    assert!(
                        ui.item_rect_size()[1] <= ui.text_line_height(),
                        "Weapon text must not create tall rows"
                    );
                });
            ui.render();
        }
        let mut heights = [0.0; 2];
        let mut line_step = 0.0;
        for (index, show) in [false, true].into_iter().enumerate() {
            let config = Settings {
                show_weapons: show,
                ..config.clone()
            };
            let mut view = View::default();
            for _ in 0..3 {
                let ui = context.frame();
                Window::new(&format!("weapon-visibility-{index}"))
                    .position([0.0, 0.0], Condition::Always)
                    .size([400.0, 400.0], Condition::Always)
                    .build(&ui, || {
                        let start = ui.cursor_screen_pos()[1];
                        highlights(&ui, &report, &mut view, &config, 6, Language::English);
                        heights[index] = ui.cursor_screen_pos()[1] - start;
                        line_step = ui.text_line_height() + ui.clone_style().item_spacing[1];
                    });
                ui.render();
            }
        }
        assert!(
            (heights[1] - heights[0] - line_step).abs() <= 0.1,
            "Hiding weapons must remove the entire second line: {heights:?}, step {line_step}"
        );
        let mut translated_view = View {
            team: Some(Team::Green),
            select_team: true,
            ..Default::default()
        };
        for language in [Language::Spanish, Language::English, Language::Spanish] {
            for _ in 0..3 {
                let ui = context.frame();
                Window::new("temporary-language")
                    .size([400.0, 400.0], Condition::Always)
                    .build(&ui, || {
                        highlights(&ui, &report, &mut translated_view, &config, 6, language);
                    });
                ui.render();
                assert_eq!(
                    translated_view.team,
                    Some(Team::Green),
                    "Temporary translation must preserve the selected team"
                );
            }
        }
        let mut view = View {
            report: Some(Arc::clone(&report)),
            ..Default::default()
        };
        let mut w = WindowSettings::for_kind(SUMMARY);
        w.visible = true;
        let mut actions = vec![];
        let mut popup_seen = false;
        for frame in 0..5 {
            context.io_mut().mouse_pos = [w.position[0] + 8.0, w.position[1] + 8.0];
            context.io_mut().mouse_down[1] = frame == 2;
            let ui = context.frame();
            render_window(
                &ui,
                SUMMARY,
                "context-test",
                &mut view,
                &mut w,
                &config,
                &[Arc::clone(&report)],
                &Message::default(),
                Language::English,
                &mut actions,
                false,
            );
            Window::new("probe###context-test").build(&ui, || {
                if let Some(_popup) = ui.begin_popup("window_menu") {
                    popup_seen = true;
                }
            });
            ui.render();
        }
        assert!(
            popup_seen,
            "Right-clicking the title must open the context menu"
        );
        context.io_mut().mouse_down[1] = false;
        let older_report = Arc::new(Report {
            path: "20261019-211400.zevtc".into(),
            file_name: "20261019-211400.zevtc".into(),
            fight: Fight::default(),
            parse_time: Default::default(),
            live: None,
        });
        let history = [Arc::clone(&report), Arc::clone(&older_report)];
        let mut history_hover = [0.0; 2];
        let mut log_click = [0.0; 2];
        let mut selected_paths = vec![];
        for frame in 0..40 {
            context.io_mut().mouse_pos = match frame {
                0..=4 => [w.position[0] + 8.0, w.position[1] + 8.0],
                5..=14 => history_hover,
                15..=31 => log_click,
                _ => [1900.0, 1060.0],
            };
            context.io_mut().mouse_down[1] = frame == 2;
            context.io_mut().mouse_down[0] = matches!(frame, 18 | 26 | 34);
            let ui = context.frame();
            let mut history_actions = vec![];
            render_window(
                &ui,
                SUMMARY,
                "history-stays-open",
                &mut view,
                &mut w,
                &config,
                &history,
                &Message::default(),
                Language::English,
                &mut history_actions,
                false,
            );
            for action in history_actions {
                if let Action::SelectReport(selected) = action {
                    selected_paths.push(selected.path.clone());
                    view.report = Some(selected);
                }
            }
            Window::new("probe###history-stays-open").build(&ui, || {
                let popup = ui.begin_popup("window_menu");
                if matches!(frame, 20 | 28) {
                    assert!(popup.is_some(), "Selecting a log must keep the popup open");
                }
                if frame == 39 {
                    assert!(
                        popup.is_none(),
                        "Clicking outside must still close the popup"
                    );
                }
                if let Some(_popup) = popup {
                    let pos = ui.window_pos();
                    let line = ui.text_line_height() + 1.0;
                    history_hover = [pos[0] + 30.0, pos[1] + 3.0 + line * 1.5];
                    if let Some(_menu) = ui.begin_menu(Language::English.widget(Text::HistoryMenu))
                    {
                        let pos = ui.window_pos();
                        let row = if frame >= 21 { 1.0 } else { 0.0 };
                        log_click = [
                            pos[0] + 30.0,
                            pos[1] + 3.0 + line + 1.0 + line * (row + 0.5),
                        ];
                    } else if matches!(frame, 20 | 28) {
                        panic!("Selecting a log must keep the history submenu open");
                    }
                }
            });
            ui.render();
        }
        assert_eq!(
            selected_paths,
            vec![report.path.clone(), older_report.path.clone()],
            "Two successive clicks must select different logs without reopening the menu"
        );
        assert!(Arc::ptr_eq(view.report.as_ref().unwrap(), &older_report));
        context.io_mut().mouse_down[0] = false;
        let mut tabs_view = View {
            report: Some(Arc::clone(&report)),
            team: Some(Team::Red),
            ..Default::default()
        };
        let mut tabs_window = WindowSettings::for_kind(DETAILS);
        tabs_window.visible = true;
        for frame in 0..7 {
            context.io_mut().mouse_pos = [
                tabs_window.position[0] + 12.0,
                tabs_window.position[1] + 22.0,
            ];
            context.io_mut().mouse_down[0] = frame == 3;
            let ui = context.frame();
            render_window(
                &ui,
                DETAILS,
                "tab-switch",
                &mut tabs_view,
                &mut tabs_window,
                &config,
                &[Arc::clone(&report)],
                &Message::default(),
                Language::English,
                &mut vec![],
                false,
            );
            ui.render();
        }
        assert_eq!(
            tabs_view.team,
            Some(Team::Green),
            "Clicking the first color tab must change team"
        );
        tabs_view.team = Some(Team::Red);
        tabs_view.select_team = true;
        for _ in 0..3 {
            let ui = context.frame();
            render_window(
                &ui,
                DETAILS,
                "tab-switch",
                &mut tabs_view,
                &mut tabs_window,
                &config,
                &[Arc::clone(&report)],
                &Message::default(),
                Language::English,
                &mut vec![],
                false,
            );
            ui.render();
        }
        assert_eq!(
            tabs_view.team,
            Some(Team::Red),
            "Opening a team from summary must select its tab"
        );
        context.io_mut().mouse_pos = [0.0, 0.0];
        for kind in 0..3 {
            let mut view = View {
                report: Some(Arc::clone(&report)),
                ..Default::default()
            };
            let mut w = WindowSettings::for_kind(kind);
            w.visible = true;
            let mut counts = [0; 2];
            for frame in 0..5 {
                let status = if frame == 4 {
                    Message::new(Text::Loaded, vec!["10".into()])
                } else {
                    Message::default()
                };
                let ui = context.frame();
                render_window(
                    &ui,
                    kind,
                    &format!("status-{kind}"),
                    &mut view,
                    &mut w,
                    &config,
                    &[Arc::clone(&report)],
                    &status,
                    Language::English,
                    &mut vec![],
                    false,
                );
                let data = ui.render();
                if frame >= 3 {
                    counts[frame - 3] = data.total_vtx_count;
                }
            }
            assert_eq!(
                counts[0], counts[1],
                "Loaded-history status must not draw extra text in window {kind}"
            );
        }
        for kind in 0..3 {
            let mut view = View {
                report: Some(Arc::clone(&report)),
                ..Default::default()
            };
            let mut w = WindowSettings::for_kind(kind);
            w.visible = true;
            w.locked = true;
            w.fixed_position = true;
            let mut initial = [0.0; 2];
            let mut grip = [0.0; 2];
            for frame in 0..7 {
                if frame == 3 {
                    initial = [w.width, w.height];
                    grip = [
                        w.position[0] + w.width - 3.0,
                        w.position[1] + w.height - 3.0,
                    ];
                }
                context.io_mut().mouse_pos = if frame <= 3 {
                    grip
                } else {
                    [grip[0] + 50.0, grip[1] + 30.0]
                };
                context.io_mut().mouse_down[0] = frame == 3 || frame == 4;
                let ui = context.frame();
                render_window(
                    &ui,
                    kind,
                    &format!("resize-{kind}"),
                    &mut view,
                    &mut w,
                    &config,
                    &[Arc::clone(&report)],
                    &Message::new(Text::Loaded, vec!["10".into()]),
                    Language::English,
                    &mut vec![],
                    false,
                );
                ui.render();
            }
            assert!(
                w.width >= initial[0] + 40.0,
                "Window {kind} did not resize horizontally: {initial:?} -> {}",
                w.width
            );
            assert!(
                w.height >= initial[1] + 20.0,
                "Window {kind} did not resize vertically: {initial:?} -> {}",
                w.height
            );
            assert_eq!(
                Settings::parse(
                    &Settings {
                        windows: std::array::from_fn(|_| w.clone()),
                        ..Default::default()
                    }
                    .to_text()
                )
                .windows[kind],
                w
            );
        }
        context.io_mut().mouse_down[0] = false;
        context.io_mut().mouse_pos = [-100.0; 2];
        let large = Arc::new(Report {
            fight: Fight {
                duration_ms: 60_000,
                players: (0..180)
                    .map(|i| Player {
                        name: format!("Player {i}"),
                        profession: 1,
                        team: Some(Team::ALL[i / 60]),
                        damage: 180 - i as u64,
                        weapons: if i % 2 == 0 {
                            vec!["Spear", "Greatsword"]
                        } else {
                            vec![]
                        },
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            },
            ..(*report).clone()
        });
        let mut view = View {
            team: Some(Team::Red),
            ..Default::default()
        };
        let mut max_scroll = 0.0;
        let mut cached_rows = std::ptr::null();
        for frame in 0..6 {
            let ui = context.frame();
            Window::new("large-scroll")
                .size([300.0, 160.0], Condition::Always)
                .position([10.0, 10.0], Condition::Always)
                .build(&ui, || {
                    highlights(&ui, &large, &mut view, &config, 6, Language::English);
                    let rows = &view.highlights.as_ref().unwrap().rows;
                    assert_eq!(rows.len(), 60);
                    if frame > 0 {
                        assert_eq!(
                            cached_rows,
                            rows.as_ptr(),
                            "Stable frames must reuse the cache"
                        );
                    }
                    cached_rows = rows.as_ptr();
                    if frame >= 2 {
                        max_scroll = ui.scroll_max_y();
                        assert!(max_scroll > 1000.0);
                    }
                    if frame == 2 {
                        ui.set_scroll_y(ui.scroll_max_y());
                    }
                    if frame >= 4 {
                        assert!((ui.scroll_y() - max_scroll).abs() <= 1.0);
                        assert!(
                            ui.is_item_visible(),
                            "The last player must render after scrolling to the bottom"
                        );
                    }
                });
            let data = ui.render();
            assert!(
                data.total_vtx_count < 4000,
                "Offscreen rows must not generate geometry: {}",
                data.total_vtx_count
            );
        }
        let all_healers = Arc::new(Report {
            fight: Fight {
                pov_team: Some(Team::Red),
                pov_subgroup: Some(1),
                healing_coverage: HealingCoverage::Partial,
                players: large
                    .fight
                    .players
                    .iter()
                    .take(60)
                    .cloned()
                    .map(|mut p| {
                        p.squad_member = true;
                        p.subgroup = Some(1);
                        p.healing = 100;
                        p.healing_coverage = HealingCoverage::Partial;
                        p
                    })
                    .collect(),
                ..Default::default()
            },
            live: Some(app::LiveStatus {
                limited: false,
                finished: false,
            }),
            ..(*large).clone()
        });
        assert_eq!(healing_rows(&all_healers.fight, 0, false).len(), 60);
        let mut healing_view = View {
            report: Some(all_healers),
            ..Default::default()
        };
        let mut healing_window = WindowSettings::for_kind(HEALING);
        healing_window.width = 300.0;
        healing_window.height = 116.0;
        healing_window.visible = true;
        for frame in 0..6 {
            let ui = context.frame();
            render_window(
                &ui,
                HEALING,
                "all-healers",
                &mut healing_view,
                &mut healing_window,
                &config,
                &[],
                &Message::default(),
                Language::English,
                &mut vec![],
                false,
            );
            Window::new("inspect###all-healers").build(&ui, || {
                if frame >= 2 {
                    assert!(
                        ui.scroll_max_y() > 500.0,
                        "All 60 healers must be scrollable"
                    );
                    ui.set_scroll_y(ui.scroll_max_y());
                }
                if frame >= 4 {
                    assert!((ui.scroll_y() - ui.scroll_max_y()).abs() <= 1.0);
                }
            });
            ui.render();
        }
        let live_report = Arc::new(Report {
            live: Some(app::LiveStatus {
                limited: false,
                finished: false,
            }),
            fight: Fight {
                pov_team: Some(Team::Own),
                duration_ms: 10_000,
                players: vec![
                    Player {
                        team: Some(Team::Own),
                        name: "Ally".into(),
                        damage: 1000,
                        ..Default::default()
                    },
                    Player {
                        team: Some(Team::Red),
                        name: "Enemy".into(),
                        damage: 2000,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ..(*large).clone()
        });
        let ui = context.frame();
        Window::new("live-own-team")
            .size([400.0, 300.0], Condition::Always)
            .build(&ui, || {
                let mut actions = vec![];
                overview(&ui, &live_report, 0.75, Language::English, &mut actions);
                let mut live_view = View {
                    team: Some(Team::Red),
                    ..Default::default()
                };
                highlights(
                    &ui,
                    &live_report,
                    &mut live_view,
                    &config,
                    6,
                    Language::English,
                );
                assert_eq!(live_view.team, Some(Team::Own));
                let cache = live_view.highlights.as_ref().unwrap();
                assert_eq!(cache.rows, vec![0]);
                assert_eq!(cache.values[0][0], "100");
            });
        ui.render();
        let mut heading_mouse = [-100.0; 2];
        for frame in 0..4 {
            context.io_mut().mouse_pos = heading_mouse;
            let ui = context.frame();
            Window::new("compact-heading")
                .size([400.0, 150.0], Condition::Always)
                .position([800.0, 50.0], Condition::Always)
                .build(&ui, || {
                    for language in [
                        Language::Spanish,
                        Language::English,
                        Language::French,
                        Language::German,
                    ] {
                        assert_eq!(
                            language.text(metric_heading_label(Text::DownContribution)),
                            "down c"
                        );
                        let columns = metric_columns(
                            &ui,
                            DETAILS,
                            8,
                            &[[String::new(), String::new(), String::new(), "123".into()]],
                            language,
                        );
                        assert_eq!(columns.len(), 1);
                        assert!(
                            (columns[0].width - (ui.calc_text_size("down c")[0] + 5.0)).abs()
                                < 0.01
                        );
                    }
                    let columns = [MetricColumn {
                        metric: 3,
                        width: 70.0,
                    }];
                    let pos = ui.cursor_screen_pos();
                    heading_mouse = [pos[0] + ui.content_region_avail()[0] - 20.0, pos[1] + 3.0];
                    let hovered = metric_heading(
                        &ui,
                        "Team",
                        DETAILS,
                        &columns,
                        &mut view,
                        Language::English,
                    );
                    if frame >= 2 {
                        assert_eq!(
                            hovered,
                            Some(3),
                            "The compact heading must expose the full metric on hover"
                        );
                    }
                });
            ui.render();
        }
    }
}
