use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    bridge,
    i18n::{self, Language, tr},
    models::{AreaChallenge, AreaClaimResult, AreaDrop, AreaPositionSample, AreaWallet},
    util::OptionValueOrElseExt,
};

use super::{
    AreaClaimArgs, AreaDropArgs, FanLoadingState, Skeleton, UrlArgs, open_area_game,
    refresh_fan_area,
};

#[derive(Clone, Debug)]
struct NearestPoint {
    drop_id: String,
    distance_meters: f64,
    accuracy_meters: f64,
    bearing_degrees: f64,
}

fn find_drop(area: RwSignal<Option<AreaWallet>>, id: &str) -> Option<AreaDrop> {
    area.get()
        .and_then(|wallet| wallet.drops.into_iter().find(|drop| drop.id.as_str() == id))
}

fn live(area: RwSignal<Option<AreaWallet>>, id: &str) -> bool {
    area.get().is_some_and(|wallet| {
        wallet
            .drops
            .iter()
            .find(|drop| drop.id.as_str() == id)
            .is_some_and(|drop| drop.active && !drop.full)
            || wallet.live_drops.iter().any(|drop| drop.id == id)
    })
}

fn claimed(area: RwSignal<Option<AreaWallet>>, id: &str) -> bool {
    area.get().is_some_and(|wallet| {
        wallet.claims.iter().any(|claim| claim.drop_id == id)
            || wallet
                .drops
                .iter()
                .find(|drop| drop.id.as_str() == id)
                .is_some_and(|drop| drop.claimed)
    })
}

fn clue(drop: &AreaDrop) -> String {
    match i18n::current() {
        Language::Pl => drop.clue.pl.clone(),
        Language::En => drop.clue.en.clone(),
    }
}

fn distance_meters(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let radius = 6_371_000.0_f64;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin().powi(2);
    radius * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

fn bearing_degrees(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let start = lat1.to_radians();
    let target = lat2.to_radians();
    let delta_lng = (lng2 - lng1).to_radians();
    let y = delta_lng.sin() * target.cos();
    let x = start.cos() * target.sin() - start.sin() * target.cos() * delta_lng.cos();
    (y.atan2(x).to_degrees() + 360.0) % 360.0
}

fn direction_label(degrees: f64) -> String {
    const DIRECTIONS: [(&str, &str); 8] = [
        ("↑", "direction_north"),
        ("↗", "direction_northeast"),
        ("→", "direction_east"),
        ("↘", "direction_southeast"),
        ("↓", "direction_south"),
        ("↙", "direction_southwest"),
        ("←", "direction_west"),
        ("↖", "direction_northwest"),
    ];
    let sector = (((degrees + 22.5) / 45.0).floor() as usize) % DIRECTIONS.len();
    let (arrow, key) = DIRECTIONS[sector];
    i18n::format(
        "direction_to_point",
        &[arrow.to_owned(), tr(key).to_owned()],
    )
}

fn distance_label(value: f64) -> String {
    if value < 1_000.0 {
        i18n::format("approximate_distance_meters", &[format!("{value:.0}")])
    } else {
        i18n::format(
            "approximate_distance_kilometers",
            &[format!("{:.1}", value / 1_000.0)],
        )
    }
}

fn open_route(drop: &AreaDrop, error: RwSignal<Option<String>>) {
    let url = format!(
        "https://www.google.com/maps/dir/?api=1&destination={:.4},{:.4}",
        drop.approximate_lat, drop.approximate_lng
    );
    spawn_local(async move {
        if let Err(message) = bridge::invoke_unit("open_external_url", &UrlArgs { url: &url }).await
        {
            error.set(Some(message));
        }
    });
}

#[component]
pub(super) fn AreaGameScreen(
    area: RwSignal<Option<AreaWallet>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let selected = RwSignal::new(None::<String>);
    let nearest = RwSignal::new(None::<NearestPoint>);
    let locating = RwSignal::new(false);
    let claiming = RwSignal::new(false);
    let result = RwSignal::new(None::<AreaClaimResult>);

    Effect::new(move |_| {
        let wallet = area.get();
        let selected_is_valid = selected.get_untracked().is_some_and(|id| {
            wallet
                .as_ref()
                .is_some_and(|wallet| wallet.drops.iter().any(|drop| drop.id.as_str() == id))
        });
        if selected_is_valid {
            return;
        }
        let initial = wallet.as_ref().and_then(|wallet| {
            wallet
                .drops
                .iter()
                .find(|drop| drop.active && !drop.full)
                .or_else(|| wallet.drops.first())
                .map(|drop| drop.id.clone())
        });
        selected.set(initial);
    });

    let refresh = move |_| refresh_fan_area(area, loading, error);
    let locate = move |_| {
        if locating.get_untracked() {
            return;
        }
        let active = area
            .get_untracked()
            .map(|wallet| {
                wallet
                    .drops
                    .into_iter()
                    .filter(|drop| drop.active && !drop.full)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if active.is_empty() {
            error.set(Some(tr("no_active_area_points_now").to_owned()));
            return;
        }

        locating.set(true);
        spawn_local(async move {
            match bridge::current_position().await {
                Ok(position) => {
                    let closest = active
                        .into_iter()
                        .map(|drop| {
                            let distance = distance_meters(
                                position.lat,
                                position.lng,
                                drop.approximate_lat,
                                drop.approximate_lng,
                            );
                            (drop, distance)
                        })
                        .min_by(|left, right| left.1.total_cmp(&right.1));
                    if let Some((drop, distance_meters)) = closest {
                        selected.set(Some(drop.id.clone()));
                        nearest.set(Some(NearestPoint {
                            drop_id: drop.id,
                            distance_meters,
                            accuracy_meters: position.accuracy,
                            bearing_degrees: bearing_degrees(
                                position.lat,
                                position.lng,
                                drop.approximate_lat,
                                drop.approximate_lng,
                            ),
                        }));
                    }
                }
                Err(message) => error.set(Some(message)),
            }
            locating.set(false);
        });
    };

    let verify = move |_| {
        if claiming.get_untracked() {
            return;
        }
        let Some(drop_id) = selected.get_untracked() else {
            error.set(Some(tr("select_an_active_point_first").to_owned()));
            return;
        };
        if !live(area, &drop_id) {
            error.set(Some(tr("select_an_active_point_first").to_owned()));
            return;
        }
        claiming.set(true);
        result.set(None);
        spawn_local(async move {
            let outcome = async {
                let challenge = bridge::invoke::<AreaChallenge, _>(
                    "fan_area_challenge",
                    &AreaDropArgs { drop_id: &drop_id },
                )
                .await?;
                let samples: Vec<AreaPositionSample> = bridge::collect_location_samples(
                    challenge.min_samples,
                    challenge.max_samples,
                    challenge.min_duration_ms,
                )
                .await?;
                bridge::invoke::<AreaClaimResult, _>(
                    "fan_area_claim",
                    &AreaClaimArgs {
                        drop_id: &drop_id,
                        challenge: &challenge.challenge,
                        samples: &samples,
                    },
                )
                .await
            }
            .await;
            match outcome {
                Ok(value) => {
                    result.set(Some(value));
                    refresh_fan_area(area, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
            claiming.set(false);
        });
    };

    view! {
        <section class="screen fan-screen area-screen area-native-screen">
            <header class="screen-title">
                <p class="eyebrow">{tr("area_in_the_app")}</p>
                <h2>{tr("find_a_point_in_your_city")}</h2>
                <p>{tr("choose_an_active_point_and_follow_the")}</p>
            </header>
            <Show when=move || !loading.get().area fallback=move || view! { <Skeleton rows=3 /> }>
                {move || area.get().map(|wallet| {
                    let claimed_count = wallet.claims.len() as u32;
                    let total = wallet.collection_size.max(1);
                    let percent = (claimed_count.saturating_mul(100) / total).min(100);
                    let live_count = wallet.drops.iter().filter(|drop| drop.active && !drop.full).count();
                    let collection_size = wallet.collection_size;
                    let reward_credits = wallet.reward_credits;
                    let map_drops = wallet.drops.clone();
                    view! {
                        <div class="area-hero-card compact">
                            <div><p class="eyebrow">{tr("collection_progress")}</p><h3>{format!("{claimed_count} / {collection_size}")}</h3></div>
                            <strong>{i18n::format("reward_credits_credits", &[reward_credits.to_string()])}</strong>
                            <div class="area-progress" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow=percent><span style=format!("width:{percent}%")></span></div>
                            <div class="area-stats"><span>{i18n::format("live_count_active_points", &[live_count.to_string()])}</span><span>{i18n::format("community_percent_community", &[wallet.community.percent.round().to_string()])}</span></div>
                        </div>

                        <div class="area-native-map" aria-label=tr("area_game_tab")>
                            <div class="area-map-grid" aria-hidden="true"></div>
                            <For
                                each=move || map_drops.clone()
                                key=|drop| drop.id.clone()
                                children=move |drop| {
                                    let id = drop.id.clone();
                                    let live_id = id.clone();
                                    let claimed_id = id.clone();
                                    let selected_id = id.clone();
                                    let pressed_id = id.clone();
                                    let click_id = id;
                                    let aria = format!("{} {}", drop.number, drop.city);
                                    let style = format!("left:{}%;top:{}%", drop.map_x, drop.map_y);
                                    let number = drop.number;
                                    view! {
                                        <button
                                            type="button"
                                            class="area-native-marker"
                                            class:is-live=move || live(area, &live_id)
                                            class:is-claimed=move || claimed(area, &claimed_id)
                                            class:is-selected=move || selected.get().as_deref() == Some(selected_id.as_str())
                                            style=style
                                            aria-label=aria
                                            aria-pressed=move || selected.get().as_deref() == Some(pressed_id.as_str())
                                            on:click=move |_| selected.set(Some(click_id.clone()))
                                        >
                                            <span aria-hidden="true">"⌖"</span><small>{number}</small>
                                        </button>
                                    }
                                }
                            />
                        </div>

                        {move || selected.get().and_then(|id| find_drop(area, &id)).map(|drop| {
                            let drop_id = drop.id.clone();
                            let is_live = live(area, &drop_id);
                            let is_claimed = claimed(area, &drop_id);
                            let status = if is_claimed { tr("claimed_area_point") } else if is_live { tr("active_area_point") } else { tr("inactive_area_point") };
                            let nearest_copy = nearest.get().filter(|value| value.drop_id == drop_id);
                            let city = drop.city.clone();
                            let region = drop.region.clone();
                            let number = drop.number.clone();
                            let drop_clue = clue(&drop);
                            let city_for_distance = city.clone();
                            let route_drop = drop.clone();
                            let route_id = drop_id.clone();
                            let claim_id = drop_id;
                            view! {
                                <article class="area-target-card" class:is-live=is_live>
                                    <div class="area-target-heading"><span>{number}</span><div><p class="eyebrow">{status}</p><h3>{city}</h3><small>{region}</small></div></div>
                                    <p>{drop_clue}</p>
                                    {nearest_copy.map(|point| view! {
                                        <div class="area-distance">
                                            <strong>{i18n::format("you_are_about_distance_from_city", &[distance_label(point.distance_meters), city_for_distance.clone()])}</strong>
                                            <span>{direction_label(point.bearing_degrees)}</span>
                                            <small>{i18n::format("location_accuracy_value", &[format!("{:.0}", point.accuracy_meters)])}</small>
                                        </div>
                                    })}
                                    <div class="area-target-actions">
                                        <button
                                            type="button"
                                            class="ghost"
                                            disabled=move || !live(area, &route_id)
                                            on:click=move |_| open_route(&route_drop, error)
                                        >
                                            {tr("open_route_start")}
                                        </button>
                                        <button
                                            type="button"
                                            class="primary"
                                            disabled=move || !live(area, &claim_id) || claiming.get()
                                            on:click=verify
                                        >
                                            {move || if claiming.get() { tr("verifying_location") } else { tr("verify_location_and_win") }}
                                        </button>
                                    </div>
                                </article>
                            }.into_any()
                        }).value_or_else(|| view! { <div class="empty-state"><strong>{tr("select_an_active_point_first")}</strong></div> }.into_any())}

                        {move || result.get().and_then(|value| {
                            let AreaClaimResult {
                                already_claimed,
                                collectible,
                                reward_credits_awarded,
                                ..
                            } = value;
                            collectible.map(|collectible| {
                                let title = if already_claimed {
                                    tr("area_point_already_won")
                                } else {
                                    tr("area_point_won")
                                };
                                let message = if already_claimed {
                                    i18n::format(
                                        "area_reward_already_present",
                                        std::slice::from_ref(&collectible.track),
                                    )
                                } else {
                                    i18n::format(
                                        "area_reward_added",
                                        &[
                                            collectible.track.clone(),
                                            reward_credits_awarded.to_string(),
                                        ],
                                    )
                                };
                                view! {
                                    <div class="area-win-card" role="status">
                                        <p class="eyebrow">{title}</p>
                                        <h3>{collectible.track}</h3>
                                        <p>{message}</p>
                                        <small>{format!(
                                            "{} · {} · #{}",
                                            collectible.city,
                                            collectible.edition,
                                            collectible.number,
                                        )}</small>
                                    </div>
                                }
                            })
                        })}

                        <div class="area-actions area-native-actions">
                            <button type="button" class="primary" disabled=move || locating.get() on:click=locate>{move || if locating.get() { tr("locating_you") } else { tr("locate_nearest_point") }}</button>
                            <button type="button" class="ghost" on:click=refresh disabled=move || loading.get().area>{tr("refresh_progress")}</button>
                            <button type="button" class="ghost" on:click=move |_| open_area_game(error)>{tr("open_full_area_game")}</button>
                        </div>
                        <p class="security-note">{tr("area_location_privacy")}</p>
                        {wallet.drops.iter().all(|drop| !drop.active || drop.full).then(|| view! {
                            <p class="inline-note">{tr("no_active_area_points_now")}</p>
                        })}
                    }.into_any()
                }).value_or_else(|| view! { <div class="empty-state"><strong>{tr("area_is_temporarily_unavailable")}</strong><p>{tr("refresh_the_data_or_open_the_full")}</p><button class="primary" on:click=move |_| open_area_game(error)>{tr("open_area")}</button></div> }.into_any())}
            </Show>
        </section>
    }
}
