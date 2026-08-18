const THOMANN_QUAD_CORTEX_URL: &str = "https://www.thomann.pl/neural_dsp_quad_cortex.htm?offid=1&affid=4979&subid=signal&subid2=gear";
const THOMANN_HOME_URL: &str =
    "https://www.thomann.pl/?offid=1&affid=4979&subid=signal&subid2=shop";

#[component]
fn FanAffiliateGear(error: RwSignal<Option<String>>) -> impl IntoView {
    let copy = i18n::affiliate::gear(i18n::current());

    view! {
        <section class="affiliate-gear" aria-label=copy.section_aria>
            <header class="affiliate-gear-heading">
                <div>
                    <p class="eyebrow">{copy.eyebrow}</p>
                    <h3>{copy.title}</h3>
                    <p>{copy.intro}</p>
                </div>
                <span class="affiliate-partner-pill">"THOMANN · AFFILIATE"</span>
            </header>

            <article class="affiliate-gear-card">
                <div class="affiliate-gear-mark" aria-hidden="true"><span>"QC"</span></div>
                <div class="affiliate-gear-body">
                    <small>{copy.used_live}</small>
                    <strong>"Neural DSP Quad Cortex"</strong>
                    <p>{copy.product_note}</p>
                    <ExternalLink
                        url=THOMANN_QUAD_CORTEX_URL.to_owned()
                        label=copy.product_cta
                        error=error
                    />
                </div>
            </article>

            <div class="affiliate-gear-support">
                <div>
                    <strong>{copy.general_title}</strong>
                    <p>{copy.general_note}</p>
                </div>
                <ExternalLink
                    url=THOMANN_HOME_URL.to_owned()
                    label=copy.general_cta
                    error=error
                />
            </div>

            <p class="affiliate-disclosure">{copy.disclosure}</p>
        </section>
    }
}
