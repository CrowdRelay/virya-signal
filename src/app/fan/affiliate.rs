const THOMANN_QUAD_CORTEX_URL: &str = "https://signal-api.virya.music/v1/go/thomann-qc-signal";
const THOMANN_HOME_URL: &str =
    "https://signal-api.virya.music/v1/go/thomann-shop-signal";

#[component]
fn FanAffiliateGear(error: RwSignal<Option<String>>) -> impl IntoView {

    view! {
        <section class="affiliate-gear" aria-label=tr("affiliate_section_aria")>
            <header class="affiliate-gear-heading">
                <div>
                    <p class="eyebrow">{tr("affiliate_eyebrow")}</p>
                    <h3>{tr("affiliate_title")}</h3>
                    <p>{tr("affiliate_intro")}</p>
                </div>
                <span class="affiliate-partner-pill">{tr("affiliate_partner_pill")}</span>
            </header>

            <article class="affiliate-gear-card">
                <div class="affiliate-gear-mark" aria-hidden="true"><span>"QC"</span></div>
                <div class="affiliate-gear-body">
                    <small>{tr("affiliate_used_live")}</small>
                    <strong>"Neural DSP Quad Cortex"</strong>
                    <p>{tr("affiliate_product_note")}</p>
                    <ExternalLink
                        url=THOMANN_QUAD_CORTEX_URL.to_owned()
                        label=tr("affiliate_product_cta")
                        error=error
                    />
                </div>
            </article>

            <div class="affiliate-gear-support">
                <div>
                    <strong>{tr("affiliate_general_title")}</strong>
                    <p>{tr("affiliate_general_note")}</p>
                </div>
                <ExternalLink
                    url=THOMANN_HOME_URL.to_owned()
                    label=tr("affiliate_general_cta")
                    error=error
                />
            </div>

            <p class="affiliate-disclosure">{tr("affiliate_disclosure")}</p>
        </section>
    }
}
