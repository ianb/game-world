# Image generation APIs for 2D game assets: native transparency, reference images, training, hosting, licensing (non-Google)

Researched 2026-10-22. Scope: every major provider except Google (covered elsewhere). Pixel-art-specific tools (Retro Diffusion, PixelLab) are mentioned only for comparison. Prices are USD list prices as published on the date fetched; several providers load prices client-side and could not be scraped, which is noted where it applies.

Short orientation for the report writer:

- Three APIs produce a real alpha channel at generation time without a removal step: OpenAI gpt-image (`background: "transparent"`, GA on gpt-image-2.5, preview on gpt-image-2), Ideogram (dedicated `ideogram-4-transparent` / `ideogram-3-transparent` endpoints plus a `background` field on 4.5), and Leonardo (`transparency: "foreground_only"`, SDXL-era models only). Recraft, Stability, Black Forest Labs (FLUX), Adobe Firefly and Midjourney do not; they offer background removal as a separate call or not at all.
- Native alpha is not clean alpha. Forum measurements on gpt-image-2 show opaque regions at alpha 252–254 and thin grey/white halos; a one-pass normalize-and-defringe step in Node is still needed.
- Reference-image conditioning is now standard: gpt-image edits take up to 16 input images, FLUX.2 [pro] up to 8, Ideogram 4.5 up to 5, Recraft style references up to 10, Ideogram 3.0 exactly 1 character reference. No source gave controlled drift measurements across dozens of generations.
- Background removal from Node without a GPU is cheap: Replicate rembg/BiRefNet listings run $0.0003–$0.026 per image, Recraft $0.01, Stability $0.05, fal BiRefNet v2 with a Node client. No source tested these on pixel art; the pixel-art-specific advice is to threshold alpha to 0/255 and defringe rather than rely on matting.

## Key question 1: Native alpha / transparent output by provider

### Takeaway

OpenAI, Ideogram and Leonardo are the only providers found with a documented generation-time alpha channel. OpenAI's is the most flexible (any size, edits endpoint, up to 16 reference inputs) but is "preview" on gpt-image-2 and ships with slightly-off alpha and halos; Ideogram's is a dedicated endpoint returning "PNGs with alpha" at up to 2k native (4k/8k by upscale) and is the only one with an exact-price `dry_run` quote; Leonardo's is tied to older SDXL/SD1.5 models. Recraft, Stability, BFL/FLUX, Firefly and Midjourney have no native transparent output; Recraft and Stability sell a removal call ($0.01 and $0.05).

### Cited Findings

**OpenAI gpt-image family**

- The `background` parameter on both generate and edit accepts `"transparent"`, `"opaque"` or `"auto"` (default `auto`). `gpt-image-2.5-sunburst` and `gpt-image-2.5-flare` (and their `2026-09-08` snapshots) support `opaque` and `transparent`; for `gpt-image-2` / `gpt-image-2-2026-04-21` transparent backgrounds are "in preview". When using `transparent`, `output_format` must be `png` or `webp`. — [OpenAI API reference, images](https://developers.openai.com/api/reference/cli/resources/images/index.md)
- Model list in the reference: `gpt-image-1`, `gpt-image-1-mini`, `gpt-image-1.5`, `gpt-image-2`, `gpt-image-2-2026-04-21`, `gpt-image-2.5-sunburst`, `gpt-image-2.5-sunburst-2026-09-08`, `gpt-image-2.5-flare`, `gpt-image-2.5-flare-2026-09-08`, `chatgpt-image-latest`, plus legacy `dall-e-2`/`dall-e-3`. — [OpenAI API reference, images](https://developers.openai.com/api/reference/cli/resources/images/index.md)
- `output_format`: `png`, `jpeg`, `webp`; edits default to `png`; `output_compression` 0–100 applies to webp/jpeg only. — [OpenAI API reference, images](https://developers.openai.com/api/reference/cli/resources/images/index.md)
- Sizes: standard `1024x1024`, `1536x1024`, `1024x1536`, `auto`. For `gpt-image-2` and `gpt-image-2.5-*`, arbitrary `WIDTHxHEIGHT` is allowed if width and height are divisible by 16, aspect ratio between 1:3 and 3:1, max `3840x2160`; above `2560x1440` is "experimental". — [OpenAI API reference, images](https://developers.openai.com/api/reference/cli/resources/images/index.md). The image-generation guide adds that total pixels must fall between 655,360 and 8,294,400 and no edge may exceed 3840 px. — [OpenAI image generation guide](https://developers.openai.com/api/docs/guides/image-generation)
- `quality`: `low`, `medium`, `high`, `auto`; gpt-image-2.5 models add `xhigh` and `max`. — [OpenAI API reference, images](https://developers.openai.com/api/reference/cli/resources/images/index.md)
- Latency: "complex prompts may take up to 2 minutes"; JPEG output is faster than PNG. — [OpenAI image generation guide](https://developers.openai.com/api/docs/guides/image-generation)
- The transparent-background preview for GPT-Image-2 in the API was announced by the OpenAI Developers account on August 20, 2026, pointing to a cookbook titled "Generate Transparent Image Assets for Campaigns and Presentations". A forum user confirmed `"background":"transparent"` was no longer rejected for `gpt-image-2` generate requests. — [OpenAI community thread, Aug 2026](https://community.openai.com/t/transparent-backgrounds-are-now-available-in-preview-for-gpt-image-2-in-the-api/1391541)
- The Decoder (Aug 21, 2026) reports OpenAI's guidance to leave background descriptions out of the prompt because the model may otherwise generate one, and attributes to OpenAI the claim that generating the alpha channel during creation gives better results than conventional background removal on hard cases such as glass or thin fibers. Cookbook use cases listed: product shots, slide diagrams, "design elements such as icons and stickers", merchandise artwork. — [The Decoder](https://the-decoder.com/openais-gpt-image-2-can-now-generate-images-without-a-background/)
- Edge-quality reports from the preview thread: opaque areas measured alpha 252–254 (mostly 253) rather than 255 at `quality: low` and `medium`, 1536x1024, with the suggestion to clip/normalize alpha client-side; a thin grey border outside the alpha; white-prompted sprites showed a white halo on light backgrounds; a drop shadow can carry alpha; one user's edit with a transparent input did not preserve transparency in the output; one user raised a concern that outputs carry an invisible AI watermark (unverified). — [OpenAI community thread, Aug 2026](https://community.openai.com/t/transparent-backgrounds-are-now-available-in-preview-for-gpt-image-2-in-the-api/1391541)
- A vendor guide dated Aug 14, 2026 (one week before the preview) said GPT Image 1.5 was the only model with native alpha, at about $0.08/image with aspect ratios limited to square, 3:2 and 2:3, and that "GPT Image 2 dropped that option". — [Rangy, AI Game Assets](https://rangy.ai/blog/ai-game-assets); superseded for gpt-image-2 by the [Aug 20 preview](https://community.openai.com/t/transparent-backgrounds-are-now-available-in-preview-for-gpt-image-2-in-the-api/1391541) and by the [current reference](https://developers.openai.com/api/reference/cli/resources/images/index.md) which lists transparent as GA on gpt-image-2.5.
- A second vendor (AgentBrush, undated 2026) states "gpt-image-2 does not reliably output a true alpha channel at generation time" and runs a local background-removal pass instead. — [AgentBrush](https://agentbrush.dev/blog/2d-game-asset-generation)
- An earlier OpenAI reference snapshot (as summarized in search) said gpt-image-2 returns an error for `background: transparent`; a third-party reseller (APIYI) dates the preview to 2026-08-21 and warns that in edits, transparency is produced by repainting the background away rather than cutting along the original outline, so subject details can change. — [APIYI docs](https://docs.apiyi.com/en/live/2026-08/gpt-image-2-transparent-background) (reseller, secondary)
- Pricing, per 1M tokens (official): gpt-image-2.5-sunburst/flare and gpt-image-2: image input $8.00, image output $30.00, cached image input $2.00, text input $5.00; gpt-image-1.5: image in $8.00, image out $32.00, text in $5.00, text out $10.00; gpt-image-1-mini: image in $2.50, image out $8.00; gpt-image-1: image in $10.00, image out $40.00. Cached input rates for gpt-image-2/2.5 apply only to the Responses API. — [OpenAI pricing docs](https://developers.openai.com/api/docs/pricing)
- Per-image equivalents (third-party, August 2026 snapshot): GPT Image 2 about $0.05 at 2K; GPT Image 1.5 about $0.08; gpt-image-1-mini low about $0.005. — [Rangy](https://rangy.ai/blog/ai-game-assets); [techsy roundup](https://techsy.io/fr/blog/best-ai-image-providers). gpt-image-1 per-image range $0.011–$0.25 by quality/size. — [MindStudio](https://www.mindstudio.ai/blog/what-is-gpt-image-1-openai) (secondary)
- Rate limits for gpt-image-1: Tier 1 100,000 TPM and 5 images/min, rising to 8,000,000 TPM and 250 images/min at Tier 5. — [OpenAI model page via search](https://developers.openai.com/api/docs/models/gpt-image-1)

**Ideogram**

- Ideogram's v2 API has dedicated transparent generation endpoints: `POST https://api.ideogram.ai/v2/image/generate/ideogram-4-transparent` and `.../ideogram-3-transparent`. Both state images are "delivered as PNGs with alpha". — [Ideogram 4.0 transparent](https://developer.ideogram.ai/v2/api-reference/images/generate/ideogram-4-transparent.md); [Ideogram 3.0 transparent](https://developer.ideogram.ai/v2/api-reference/images/generate/ideogram-3-transparent.md)
- `ideogram-4-transparent` parameters: `prompt` (natural language or structured Ideogram 4.0 JSON prompt; a JSON prompt's background description is replaced with a transparent background), `magic_prompt` auto/on/off, `aspect_ratio` (auto, 1x4, 1x3, 1x2, 9x16, 10x16, 2x3, 3x4, 4x5, 1x1, 5x4, 4x3, 3x2, 16x10, 16x9, 2x1, 3x1, 4x1), `output_resolution` 1k/2k/4k/8k as a total pixel budget ("tiers above 2k are produced by upscaling after generation"), `rendering_speed` turbo/default/quality, `num_images`, `seed`, `async` + `webhook_url` (Ed25519-signed), and a `dry_run=true` query parameter that returns a `PriceQuote` without generating or billing. The page does not document style or reference-image inputs or `color_palette` on this endpoint. — [Ideogram 4.0 transparent](https://developer.ideogram.ai/v2/api-reference/images/generate/ideogram-4-transparent.md)
- `ideogram-3-transparent` adds `negative_prompt` and `upscale_factor` x1/x2/x4; `aspect_ratio` cannot be combined with `resolution`. 402/429 responses carry `reject_reason` values such as `insufficient_funds`, `daily_limit`, `inflight_limit` (with `max_inflight_requests`). — [Ideogram 3.0 transparent](https://developer.ideogram.ai/v2/api-reference/images/generate/ideogram-3-transparent.md)
- Ideogram 4.5 (`/v2/image/generate/ideogram-4-5`) has a `background` field accepting `transparent` (alpha channel), `opaque`, or `auto`, alongside `images` (up to 5), `mask`, `size`, `quality`, `num_images`, `seed`. Billing is tiered by output pixels: up to 1024x1024 bills as 1K, larger as 2K; `auto` size bills as 2K. Dollar amounts are not on the page; use `dry_run`. — [Ideogram 4.5 generate](https://developer.ideogram.ai/v2/api-reference/images/generate/ideogram-4-5.md)
- Pricing page: "1¢ per credit", default rate limit "10 in-flight requests", models listed Ideogram Vector, 4.5, 4.0, 3.0, P-Image-Ideogram, Custom Image Models; the per-endpoint table is loaded dynamically and was not scrapeable. — [Ideogram pricing](http://ideogram.ai/pricing/?pricing_tab=api)
- Ideogram 3.0 per-image rates (from the pricing page as of its Aug 6, 2025 revision, via search): Turbo $0.03, Default $0.06, Quality $0.09; with character reference $0.10 / $0.15 / $0.20. Rates apply to Generate, Remix, Edit, Reframe and Replace Background. — [Ideogram API pricing](https://ideogram.ai/api-pricing/); repeated by [Puter](https://developer.puter.com/tutorials/ideogram-api-pricing/)

**Leonardo.ai**

- The generations endpoint takes `"transparency": "foreground_only"`, which "generates the foreground and removes the background"; output is "a PNG file with no background". Documented example uses Leonardo Kino XL (`modelId` `aa77f04e-3eec-4034-9c07-d0f619684628`), `alchemy: false`, 512x512, plus an element (Simple Flat Illustration, `akUUID` `5f3e58d8-7af3-4d5b-92e3-a3d04b9a3414`, weight 0.5). Only works with certain models and elements. — [Leonardo docs, transparency](https://docs.leonardo.ai/docs/generate-images-using-transparency)
- Leonardo's help center says Transparency works with SDXL and SD1.5 based models. — [Leonardo help](https://intercom.help/leonardo-ai/en/articles/9075772-transparency)
- At launch (March 20, 2024) Leonardo described it as "true native transparent image diffusion", not "glorified background removal", recommending Kino XL, Vision XL, AlbedoBase XL, Alchemy v2, PhotoReal v2. — [PetaPixel, Mar 2024](https://petapixel.com/2024/03/20/leonardo-ai-can-now-generate-transparent-png-images/) (older source)
- Leonardo API pricing is pay-as-you-go with cost returned per request as `apiCreditCost`; no flat per-image card is published. — [Leonardo pricing FAQ](https://docs.leonardo.ai/docs/pricing-and-plans-faq); [APIFrame guide](https://apiframe.ai/guides/leonardo-ai-api-guide) (secondary)

**Recraft (no native alpha; removal call)**

- Base URL `https://external.api.recraft.ai/v1`. `POST /images/removeBackground` "returns a transparent cutout" at $0.01. Raster generation defaults to `webp`; set `image_format: "png"`. Vector output is always SVG. The docs do not say generation itself can produce transparent backgrounds. — [Recraft llms-full.txt](https://www.recraft.ai/llms-full.txt)
- Recraft's own notes: "transparent backgrounds are currently unsupported in Upscale; workaround: upscale then re-remove BG". — [Recraft llms.txt](https://www.recraft.ai/llms.txt)
- A Recraft bug report describes generated PNGs where the "subject has a visible edge halo/alpha fringe despite prompts requesting no halo, no glow, no shadow, and transparent background". — [Recraft feedback](https://feedback.recraft.ai/bug-reports)
- Generation prices (1 USD = 1,000 API units): `recraftv4_1_flash` $0.007 (raster only, no styles), `recraftv4_1` $0.035, `recraftv4_1_pro` (2K) $0.21, `recraftv4_1_vector` $0.08, `recraftv4_1_pro_vector` $0.30, `recraftv4` $0.04, `recraftv4_pro` $0.25, `recraftv3` $0.04, `recraftv3_vector` $0.08, `recraftv2` $0.022. Tools: vectorize $0.01, remove background $0.01, crisp upscale $0.004, creative upscale $0.25. Resolution: 1K models up to 1536 px long side, 2K/Pro up to 3072; V2/V3 up to 2048x1024. Rate limits: 100 images/min and 5 requests/s per user. Curated `style` names include an `icon` style only on V2 vector models; no pixel-art style is listed. — [Recraft llms-full.txt](https://www.recraft.ai/llms-full.txt)

**Stability AI (no native alpha; removal call)**

- Credits: 1 credit = $0.01; 25 free credits on signup. Remove Background 5 credits ($0.05); Replace Background & Relight 8; Stable Image Ultra 8; Core 3; SD3.5 Large 6.5; Large Turbo 4; Medium 3.5; Flash 2.5; Inpaint 5; Outpaint 4; Erase 5; Search and Replace 5; Control Sketch 5; Control Structure 5; Style Guide 5; Style Transfer 8; Upscale Fast 2, Conservative 40, Creative 60. — [Stability pricing](https://docs.stability.ai/pricing)
- No Stability generate endpoint with a transparent-background parameter was found in the pricing or docs fetched (gap below).

**Black Forest Labs / FLUX (no native alpha found)**

- BFL's model overview and pricing pages do not mention transparency or alpha output. — [BFL help, Welcome to the forest](https://help.bfl.ai/articles/7424166364-wip-welcome-to-the-forest); [BFL pricing](https://docs.eu.bfl.ai/quick_start/pricing)
- fal's `fal-ai/flux-2-pro` schema offers `output_format` `jpeg` (default) or `png` with no transparency option. — [fal flux-2-pro](https://fal.ai/models/fal-ai/flux-2-pro/llms.txt)

**Adobe Firefly (no native alpha found)**

- Adobe community staff stated (2024) that Firefly cannot generate transparent images and suggested Adobe Express background removal as the workaround; the Firefly Services API docs found cover style/structure references but say nothing about transparent output. — [Adobe community](https://community.adobe.com/t5/adobe-firefly-ideas/p-transparent-background-or-layers/idi-p/13679242/page/4) (2023–2025 posts); [Firefly API style/structure guide](https://developer.adobe.com/firefly-services/docs/firefly-api/guides/how-tos/using-style-structure-refs)

**Midjourney**

- As of August 18, 2026 Midjourney has not released an official public API; its terms ban "automated tools to access, interact with, or generate Assets", so third-party wrappers (APIFrame, PiAPI, CometAPI) operate against the ToS and risk account bans. — [Unifically](https://unifically.com/blogs/midjourney-api); [pricepertoken](https://pricepertoken.com/midjourney-api-alternatives) (both vendor-written)

**Reve, Krea**

- Reve's API has create, edit and remix endpoints; background removal, upscaling and resizing are "optional request parameters". A competitor (Krea) blog says image generation on reve.com ends September 27, 2026 at 11:59 PM Pacific; whether the API continues is not stated. — [Reve help](https://help.reve.com/hc/en-us/articles/46837930295316); [Krea blog](https://www.krea.ai/blog/reve-sunset-2026-alternative-ai-generators) (competitor, unverified)

### Inferences

- For a headless Node pipeline the practical native-alpha choices are gpt-image-2.5 (GA, any size up to 3840x2160, edits with up to 16 references) and Ideogram 4.x transparent endpoints (dedicated, `dry_run` pricing, 1k/2k native). Leonardo's SDXL-era transparency is a weaker third option for style-consistent sprites because it is locked to older base models.
- Even with native alpha, plan a Node post-pass: clip alpha >= ~250 to 255, erode or defringe a 1-px rim, and test on a dark checkerboard. The forum measurements (alpha 253, grey halo) are exactly the artifacts that break sprite compositing on dark scenes.
- Recraft's $0.01 removeBackground and vector output are the strongest fit when the target is flat/vector-like props and icons; SVG output sidesteps alpha entirely if the renderer can rasterize SVG.

### Gaps

- OpenAI's official per-image price table (model x quality x size) was not retrievable (openai.com/api/pricing returned 403; the docs pricing page gives only per-token rates and points to a calculator). Per-image figures above are third-party.
- Ideogram's current per-image prices for 4.0/4.5 and the transparent endpoints are loaded client-side; the only way found to get exact numbers is `dry_run=true`.
- No source measured the alpha edge quality (anti-aliased vs hard) of Ideogram's or Leonardo's transparent output.
- Stability: no evidence either way of a native transparent generation parameter; its docs index was not fully read.
- Krea: no API documentation on transparency or references was found.

## Key question 2: Background removal as a fallback from Node without a GPU

### Takeaway

Hosted BiRefNet/rembg on Replicate costs a fraction of a cent per image (listings range $0.0003–$0.026 depending on hardware), fal.ai hosts BiRefNet v2 with a Node client and a Bria RMBG-2.0 variant at $0.018, and Recraft ($0.01) and Stability ($0.05) sell removal calls. No source tested any of these on pixel art; the pixel-art-specific guidance is that matting defaults (rembg thresholds 240/10, erosion 10) produce soft edges that should be thresholded to 0/255 afterward, and that keyed backgrounds leave a one-pixel colour rim that must be un-mixed, not just deleted.

### Cited Findings

- fal `fal-ai/birefnet/v2`: `model` options "General Use (Light)" (BiRefNet, recommended), "General Use (Light 2K)" (BiRefNet_lite-2K), "General Use (Heavy)", "Matting" (BiRefNet-matting), "Portrait", "General Use (Dynamic)" (256–2304 px); `operating_resolution` 1024x1024 / 2048x2048 / 2304x2304 (Dynamic only); `refine_foreground` (default true); `output_mask`; `mask_only`; `output_format` png/webp/gif. Node: `npm install @fal-ai/client`, `fal.subscribe("fal-ai/birefnet/v2", { input: { image_url } })`. The page lists price as "$0 per compute second", which reads as a placeholder. — [fal BiRefNet v2](https://fal.ai/models/fal-ai/birefnet/v2/llms.txt)
- fal's pricing page lists a free "Background Remover" tool but no BiRefNet price. — [fal pricing](https://fal.ai/pricing)
- Replicate per-run estimates (hardware in parentheses): `ilkerc/rembg` ~$0.00036 (T4); `abhisingh0909/rembg` ~$0.00032; `cjwbw/rembg` ~$0.0059 (A40, ~11 s); `851-labs/background-remover` ~$0.00051 (T4, ~3 s); `codeplugtech/background_remover` ~$0.00079; `juergengunz/birefnet` ~$0.0020 (A100 80GB); `men1scus/birefnet` ~$0.026 (A100, ~19 s). Prices "vary depending on your inputs". — [ilkerc/rembg](https://replicate.com/ilkerc/rembg), [abhisingh0909/rembg](https://replicate.com/abhisingh0909/rembg), [cjwbw/rembg](https://replicate.com/cjwbw/rembg), [juergengunz/birefnet](https://replicate.com/juergengunz/birefnet), [men1scus/birefnet](https://replicate.com/men1scus/birefnet), [codeplugtech](https://replicate.com/codeplugtech/background_remover)
- A July 2026 blog test puts Bria RMBG-2.0 (BiRefNet architecture) on fal at $0.018/image and hosted removal on Replicate "from around $0.51 per 1,000 images". — [aireiter](https://aireiter.com/blog/best-background-removal-api) (blog)
- Stability Remove Background: 5 credits = $0.05. — [Stability pricing](https://docs.stability.ai/pricing). Eden AI resells it "from $0.02/request". — [Eden AI](https://www.edenai.co/expert-models/stabilityai-background-removal)
- Recraft `removeBackground`: $0.01; input images under 20 MB, up to 16 MP, longest side 4096, shortest side at least 256 px. — [Recraft llms-full.txt](https://www.recraft.ai/llms-full.txt)
- rembg alpha matting is off by default; when on, foreground threshold 240, background threshold 10, erosion size 10 (trimap via pymatting). — [Replicate salahzsh/rembg API](https://replicate.com/salahzsh/rembg/api); [tessl rembg rubric](https://tessl.io/registry/tessl/pypi-rembg/files/evals/scenario-6/rubric.json) (third-party eval, not rembg docs)
- Pixel-art alpha thresholding: "many image editors and export tools add semi-transparent pixels along edges to make them look smoother, which looks bad in pixel art"; pixels above the threshold become fully opaque, below fully transparent. — [sprite-ai.art sharpen edges](https://www.sprite-ai.art/tools/sharpen-edges); [imag-r alpha threshold](https://imag-r.com/toolset/alphathreshold)
- TexturePacker's "Reduce border artifacts" does not remove semi-transparent border pixels; it fixes colour bleed under them. — [CodeAndWeb KB](https://www.codeandweb.com/texturepacker/knowledgebase/reduce-border-artifacts)
- Colour keying (generate on flat #FF00FF, key it out) is free but leaves a pink rim: across six assets about 0.78% of visible pixels still carried magenta, mostly a one-pixel rim on fully opaque pixels; a thin sword reached 2.47%. The fix is to sample the background colour from the border, flood inward, ramp alpha across the edge and defringe partially transparent pixels; or erode by 1 px; or key with a colour close to the asset's edge tones. Inspect on a dark background. — [Rangy](https://rangy.ai/blog/ai-game-assets) (vendor, Aug 2026)
- SpriGen, an open-source sprite generator, gets transparent backgrounds via the `rembg` library; last described "as of July 2023". — [SpriGen GitHub](https://github.com/hafiidz/sprigen)
- A ComfyUI-based handbook (Aug 9, 2026) reports a pixel-art LoRA on FLUX.2 klein-4B that outputs sprites on a transparent background directly, "which skips keying, matting, and halo cleanup", and recommends explicit background removal only when the prompt is busy. (Requires a GPU box.) — [NUILab sprites workflow](https://aigaming.nuilab.org/workflows/sprites/)

### Inferences

- For sprites that will be downscaled to a pixel grid, matting quality matters less than a deterministic post-pass: nearest-neighbour downsample, alpha threshold at ~128, optional palette quantization. Any of the sub-cent Replicate rembg endpoints is adequate as a mask source; BiRefNet "Matting" or "Heavy" is only worth paying for on painted/illustrated assets kept at full resolution.
- Generating on a flat key colour and keying in Node (sharp/jimp) is the zero-cost fallback and works when the art style has no colours near the key; the measured 0.8–2.5% rim pixels need a defringe step either way.

### Gaps

- No source tested rembg, BiRefNet or any cloud remover on pixel art or sub-64 px sprites; halo and hard-edge behaviour on those inputs is unmeasured.
- fal's BiRefNet per-image price is not published on the model page ("$0 per compute second").
- remove.bg's current API pricing was not fetched.
- Replicate's own pricing page was not fetched; per-run numbers are listing estimates.

## Key question 3: Reference-image conditioning (character, style, structure, instruction edits)

### Takeaway

Every surviving provider now accepts reference images: gpt-image edits take up to 16 images with a mask on the first; FLUX.2 [pro] up to 8; Ideogram 4.5 up to 5 (first is edited, rest are references) plus Ideogram 3.0's single masked character reference; Recraft custom styles from up to 10 images; Firefly separate style (1–100) and structure (0–100) strengths; Stability sells Control Structure/Sketch/Style calls at $0.05 each. Nobody publishes controlled drift numbers across dozens of generations, and only Ideogram 3.0 exposes a `color_palette` parameter; no API locks a pixel grid.

### Cited Findings

**OpenAI gpt-image edits**

- `image` accepts a string or an array; GPT image models accept up to 16 images, each png/webp/jpg under 50 MB. `mask` is a PNG under 4 MB with the same dimensions as `image`, fully transparent areas mark the edit region, and with multiple inputs the mask applies to the first image. `input_fidelity` `high`/`low` (default low) on gpt-image-1 and 1.5; gpt-image-1-mini low only; "for gpt-image-2, omit this parameter". — [OpenAI API reference, images](https://developers.openai.com/api/reference/cli/resources/images/index.md)
- The image guide's edit example passes four reference images; masking is "prompt-based" and the model "may not follow its shape exactly". — [OpenAI image generation guide](https://developers.openai.com/api/docs/guides/image-generation)
- AgentBrush's workflow passes an approved sprite as `reference_image_paths` on later gpt-image-2 calls; the author calls this "less tight than a fine-tune or LoRA but more stable than re-prompting from text", and reports palette locking is unreliable (quantize afterward), tile seams are not pixel-aligned, sprites below ~16x16 look like downscaled illustration, and animation frames are not mechanically consistent. — [AgentBrush](https://agentbrush.dev/blog/2d-game-asset-generation) (vendor)

**Black Forest Labs FLUX**

- FLUX.2 [pro] "supports up to 8 reference images", ~10 s turnaround, $0.03/MP text-to-image and $0.045/MP editing; [max] $0.07/MP; [flex] $0.05/MP t2i, $0.10/MP editing; [klein] 4B $0.014/image (Apache 2.0 weights), 9B $0.015/image (non-commercial weights); [dev] open weights, non-commercial, no hosted API. FLUX.1 Kontext [pro] $0.04/image (5–6 s), [max] $0.08/image. — [BFL help, Welcome to the forest](https://help.bfl.ai/articles/7424166364-wip-welcome-to-the-forest)
- BFL pricing docs: 1 credit = $0.01, same price for API and Playground; FLUX.2 is billed per megapixel with a flat first-megapixel rate; batch requests multiply cost by image count. — [BFL pricing](https://docs.eu.bfl.ai/quick_start/pricing)
- On fal, `fal-ai/flux-2-pro` is $0.03 for the first output megapixel and $0.015 per additional input or output megapixel (1 MP = 1024x1024, rounded up); 1920x1080 costs $0.045. Parameters: `prompt`, `image_size`, `seed`, `safety_tolerance` 1–5, `enable_safety_checker`, `output_format` jpeg/png, `sync_mode`. — [fal flux-2-pro](https://fal.ai/models/fal-ai/flux-2-pro/llms.txt)
- BFL's character-consistency guide shows one reference photo through three sequential edits and an "8 images, one prompt" example; tips: describe the new scene while references preserve identity, state what stays fixed ("Keep the woman's pose unchanged"), name the style/colours to carry over ("the style of Image 1"). No drift metrics. — [BFL character consistency guide](https://docs.bfl.ai/guides/usecases_editing_character_consistency)
- Third-party claims of "94.7% accuracy" or "98% identity retention" for Kontext cite no method or dataset. — [G2 listing](https://ai.g2.com/marketplace/tools/flux-kontext-2025-10-29); [RunComfy workflow](https://www.runcomfy.com/comfyui-workflows/flux-kontext-pulid-consistent-character-generation) (unreliable)

**Ideogram**

- Ideogram 3.0 generate: `character_reference_images` "currently only supports 1 character reference image" (25 MB/image, 50 MB/request, JPEG/PNG/WebP), with optional `character_reference_images_mask` (grayscale, same dimensions); `style_reference_images` (no stated max; example sends two); `style_codes` (8-char hex); `style_type` AUTO/GENERAL/REALISTIC/DESIGN/FICTION; `color_palette` (preset name or members); `seed`; `custom_model_uri` `model/<name>/version/<version>`; `enable_copyright_detection`. — [Ideogram 3.0 generate (legacy v1)](https://developer.ideogram.ai/api-reference/api-reference/generate-v3)
- Ideogram 4.5: `images` up to 5 (first edited, others references), `mask` (black = edit, white = keep; masked requests allow at most 3 other sources), `size` auto/source/exact (multiples of 32, >= 256 px, <= 2048x2048 total, AR <= 6:1), `quality` very_low/medium/high (very_low requires source images), `background` transparent/opaque/auto. — [Ideogram 4.5 generate](https://developer.ideogram.ai/v2/api-reference/images/generate/ideogram-4-5.md)
- Resellers state Ideogram 4.5 takes up to four reference images plus an optional mask (Picsart) or up to five (Leonardo). — [Picsart](https://picsart.com/api-platform/models/ideogram-4-5/advanced); [Leonardo docs](https://docs.leonardo.ai/docs/ideogram-45)
- Ideogram's own Character feature page describes a single-reference-photo workflow available in the API. — [Ideogram Character](https://ideogram.ai/features/character)

**Recraft**

- `POST /styles` creates a custom style from `image_urls`/`files` with `model`, `match` and `image_weights`; or attach `style_reference_urls` / `style_references` directly to a generation: up to 10 images, 64 MB total. `style_match` is `flexible` or `precise` on V4/4.1. Creating a style costs $0.005; attaching style references adds $0.005 per generation. `POST /images/imageToImage` takes `image_url`, `prompt`, `strength` 0–1 ($0.04). `recraftv4_styles*` models require a style. — [Recraft llms-full.txt](https://www.recraft.ai/llms-full.txt)

**Adobe Firefly Services**

- Style can come from a preset, a reference image, or both; style reference takes `source` and `strength` 1–100 (default 50). Structure reference borrows outline and depth with `strength` 0–100, available in the v3 Generate Image API. Reference images go through the Upload API (PNG/JPEG/WebP) and return an ID; cloud-storage URLs only from S3, SharePoint, Dropbox. — [Firefly style/structure how-to](https://developer.adobe.com/firefly-services/docs/firefly-api/guides/how-tos/using-style-structure-refs); [Structure reference concept](https://developer.adobe.com/firefly-services/docs/firefly-api/guides/concepts/structure-image-reference/); [Upload API](https://developer.adobe.com/firefly-services/docs/firefly-api/guides/api/upload_image/)

**Stability**

- Control Sketch, Control Structure and Style Guide are 5 credits ($0.05) each; Style Transfer 8 credits. — [Stability pricing](https://docs.stability.ai/pricing)

**Midjourney (no API; for comparison)**

- Omni Reference `--oref` launched early May 2025, covers characters, objects, vehicles, creatures; one reference image per prompt; weight `--ow` 0–1000 (default 100); each render uses twice the GPU time of a standard V7 job; `--sref` style reference weight 0–1000. Sources are secondary guides. — [AIarty](https://www.aiarty.com/midjourney-guide/midjourney-consistent-character.htm); [CometAPI guide](https://www.cometapi.com/how-to-use-omni-reference-in-midjourney-v7)

**Leonardo**

- Leonardo's own API also proxies Ideogram 4.5 with up to five reference images. — [Leonardo docs](https://docs.leonardo.ai/docs/ideogram-45)

**Practitioner consensus on consistency**

- "Generating frames is the easy part; keeping them consistent across 80+ poses is the hard part." — [Scenario blog](https://www.scenario.com/blog/ai-sprite-generator) (vendor)
- "The model reinterprets your style on every generation"; fixes: a reference image in every generation, a fixed prompt template, a locked palette, variations from one approved base. Tilesets are the weakest category (visible seams); animation details drift between frames. — [Summer Engine, June 6 2026](https://www.summerengine.com/blog/ai-2d-game-asset-generator) (vendor)

### Inferences

- For dozens of sprites of one character, the two API-native approaches are (a) gpt-image-2.5 edits with the approved base sprite plus up to 15 more references and `background: "transparent"`, or (b) FLUX.2 [pro] with up to 8 references followed by removal. Ideogram 4.5 combines 5 references with a `background: "transparent"` flag in one call, which is the shortest path on paper.
- Palette locking: only Ideogram 3.0 exposes `color_palette`; every other provider needs post-hoc quantization. No API exposes a pixel-grid constraint, which matches the handbook advice to generate large and downsample.
- Structure control (pose/silhouette) exists via Firefly structure reference and Stability Control Structure/Sketch; gpt-image and FLUX.2 rely on instruction text plus references instead.

### Gaps

- No controlled measurement of identity drift across 10+ sequential generations for any API; the only numbers found (94.7%, 98%) are unsourced marketing.
- Ideogram 4.5's style-code/character-reference semantics (does any of the 5 `images` act as a character reference with pricing uplift?) are not stated on the page fetched.
- Firefly's maximum number of reference images and its v4 model status were not found.

## Key question 4: Custom training (LoRA / fine-tune via API)

### Takeaway

Hosted LoRA training is cheap and fast for FLUX.1 (fal $0.005/step or $2.40 per 1,000 steps; Replicate's fast trainer under 2 minutes and under $2) but much pricier for FLUX.2 [dev] (fal's example $47.46 for 1,000 steps). Scenario and Spriteflow sell game-asset-specific training from 3–15 reference images. One vendor explicitly ranks consistency as LoRA > reference images > re-prompting, but no independent benchmark was found.

### Cited Findings

- fal `fal-ai/flux-lora-general-training`: $0.005 per step ("for $1 you can run approximately 200 steps"). — [fal general training](https://fal.ai/models/fal-ai/flux-lora-general-training)
- fal `fal-ai/turbo-flux-trainer`: $2.40 per 1,000-step run, scaling per step (2,000 steps = $4.80). — [fal turbo trainer](https://fal.ai/models/fal-ai/turbo-flux-trainer)
- fal `fal-ai/flux-2-trainer-v2` (FLUX.2 [dev]) cost scales with steps and reference images; the page example is $47.46 for 1 reference image at 1,000 steps. `fal-ai/flux-2-klein-4b-base-trainer` example: $9.71 for 1 reference image, 1,000 steps. — [fal flux-2-trainer-v2](https://fal.ai/models/fal-ai/flux-2-trainer-v2/edit); [fal klein 4B trainer](https://fal.ai/models/fal-ai/flux-2-klein-4b-base-trainer/edit)
- BFL hosts `flux-2-klein-9b-kv-finetuned` at the same rate as base klein-9b during public beta. — [BFL pricing](https://docs.eu.bfl.ai/quick_start/pricing)
- Replicate's fast FLUX trainer (changelog 2025-05-23): "under 2 minutes", "under $2", returns LoRA weights plus a warm runnable model. — [Replicate changelog](https://replicate.com/changelog/2025-05-23-faster-flux-trainer). Replicate H100 listed at $5.49/h ($0.001525/s) by a third party dated Sept 23, 2026. — [ModelsLab](https://modelslab.com/replicate-pricing) (secondhand)
- Scenario: "almost every custom model on Scenario is a LoRA"; a single-subject LoRA wants 5–15 images; training uploads are not billed; a `dry-run` flag returns expected credit cost. — [Scenario help](https://help.scenario.com/articles/5151772792-basics-of-model-training); [Scenario API pricing](https://docs.scenario.com/page/api-pricing). Third-party plan figures: Starter $15/mo, Pro $45/mo, Teams $75/mo. — [CostBench](https://costbench.com/software/ai-3d-generation/scenario-gg/calculator/) (unverified)
- Spriteflow.io (reviewed July 2026): upload "three to five reference images" or a preset; the platform "locks the color palette, detail level, and proportions"; author's first walk cycle took ~70 s, style lock ~2 min, ten variations ~20 min; "doesn't guarantee identical results across generations". — [PC Tech Magazine](https://pctechmag.com/2026/07/cutting-sprite-production-time-from-weeks-to-minutes-without-losing-consistency/) (affiliation undisclosed)
- Ideogram supports `custom_model_uri` and "Custom Image Models" in its API/pricing. — [Ideogram 3.0 generate](https://developer.ideogram.ai/api-reference/api-reference/generate-v3); [Ideogram pricing](http://ideogram.ai/pricing/?pricing_tab=api)
- Recraft's custom style is a lightweight alternative: $0.005 to create from up to 10 images, +$0.005 per use. — [Recraft llms-full.txt](https://www.recraft.ai/llms-full.txt)
- Practitioner ranking: reference images are "less tight than a fine-tune or LoRA but more stable than re-prompting from text". — [AgentBrush](https://agentbrush.dev/blog/2d-game-asset-generation) (vendor)
- Dataset size guidance from trainers: 10–50 photos with a trigger phrase (MuAPI, competitor), 15–30 photos (tutorial). — [MuAPI](https://muapi.ai/comparison/flux-lora-trainer); [Tech-Insider](https://tech-insider.org/train-custom-ai-image-lora-2026/) (secondary)
- A pixel-art LoRA on FLUX.2 klein-4B in ComfyUI is the production path in one Aug 2026 handbook; batches of eight, cull aggressively, generate large then reduce to the pixel grid. Needs a GPU. — [NUILab](https://aigaming.nuilab.org/workflows/sprites/)

### Inferences

- A FLUX.1-dev LoRA on fal or Replicate costs $1–3 and minutes, and inference then runs on fal/Replicate hosted LoRA endpoints, so a per-character or per-tileset LoRA is affordable even for an indie pipeline. FLUX.2 [dev] training at ~$47/run and non-commercial weights makes it a poor fit; klein-4B (Apache 2.0) at ~$10/run is the FLUX.2 option with clean licensing.
- The trade-off versus reference images is operational: a LoRA locks style across hundreds of calls with no per-call reference-megapixel charge, but needs 10–30 curated images up front and re-training whenever the style changes. gpt-image-2.5 and Ideogram 4.5 have no LoRA path, so the choice of native-alpha provider also decides this.

### Gaps

- No Astria or Leonardo training price/turnaround was found.
- fal's two FLUX.1 trainer pages disagree ~2x per step ($0.005 vs ~$0.0024); the exact `flux-lora-fast-training` page was not fetched.
- No independent study compares LoRA vs reference images on sprite consistency.

## Key question 5: Hosted inference marketplaces as the route from a container

### Takeaway

fal.ai hosts OpenAI gpt-image-2, FLUX.2 (pro/flex/klein), Ideogram 4, BiRefNet and LoRA trainers behind one key with an official Node client; Replicate bills per GPU-second with cold-start risk; Together and Fireworks have small image catalogs and are not the natural route. Ideogram's own API also proxies gpt-image-2/2.5 and Nano Banana, so an Ideogram key alone covers two native-alpha providers.

### Cited Findings

- fal pricing page lists `openai/gpt-image-2` (and `/edit`), `fal-ai/nano-banana-2`, `fal-ai/nano-banana-pro` ($0.15/image), `fal-ai/nano-banana` ($0.0398), `fal-ai/z-image/turbo` ($0.005/MP), `fal-ai/flux/schnell` ($0.003/MP), links to `fal-ai/flux-2-flex`, "Ideogram 4", a Training section and a free Background Remover; "Higher resolutions will be priced proportionally"; some models use GPU-based pricing. — [fal pricing](https://fal.ai/pricing)
- fal Node client: `@fal-ai/client`, `fal.subscribe(model, { input, logs, onQueueUpdate })` returns `result.data` and `result.requestId`. — [fal BiRefNet v2](https://fal.ai/models/fal-ai/birefnet/v2/llms.txt)
- fal `fal-ai/flux-2-pro`: $0.03 first MP, $0.015 per additional MP; `output_format` jpeg/png. — [fal flux-2-pro](https://fal.ai/models/fal-ai/flux-2-pro/llms.txt)
- Replicate "typically charges per second of GPU compute time, while Fal.ai charges per image or megapixel"; a cold start "can burn 10 to 60 seconds" of compute; Replicate serves ~50,000 models. — [apatero blog](https://www.apatero.com/blog/ai-image-generator-api-costs-2026-fal-replicate-together); [morphllm comparison](https://www.morphllm.com/comparisons/fireworks-vs-replicate) (both blogs)
- One blog claims fal is 30–50% cheaper than Replicate on the same FLUX 2 / SDXL models; an aggregator says it varies by model. — [apatero](https://www.apatero.com/blog/ai-image-generator-api-costs-2026-fal-replicate-together); [pricepertoken](https://pricepertoken.com/image)
- Fireworks is mainly a text-LLM host; cheapest listed image price $0.0105 per 1024x1024. — [pricepertoken Fireworks](https://pricepertoken.com/image/author/fireworks)
- Ideogram's v2 API exposes generate endpoints for GPT Image 2, GPT Image 2.5 Flare/Sunburst, Nano Banana 2 / 2.1 / Pro, P-Image Ideogram and Z-Image alongside its own models, plus "Generate SVG" and "Vectorizer" endpoints. — [Ideogram v2 docs index](https://developer.ideogram.ai/v2/llms.txt)
- Ideogram default rate limit: 10 in-flight requests; 402/429 responses include `inflight_limit` with `max_inflight_requests`. — [Ideogram pricing](http://ideogram.ai/pricing/?pricing_tab=api); [Ideogram 3.0 transparent](https://developer.ideogram.ai/v2/api-reference/images/generate/ideogram-3-transparent.md)
- Recraft rate limits: 100 images/min and 5 req/s per user, across all tokens. — [Recraft llms-full.txt](https://www.recraft.ai/llms-full.txt)
- OpenAI gpt-image-1 tiers: 5 images/min at Tier 1 to 250 images/min at Tier 5. — [OpenAI model page](https://developers.openai.com/api/docs/models/gpt-image-1)
- Picsart API Platform and Leonardo also resell Ideogram 4.5. — [Picsart](https://picsart.com/api-platform/models/ideogram-4-5); [Leonardo](https://docs.leonardo.ai/docs/ideogram-45)
- Runware wraps Recraft V4/4.1 and documents PNG as the alpha-capable format. — [Runware Recraft V4](https://runware.ai/docs/models/recraft-v4)

### Inferences

- From a Node container with no GPU, two keys cover nearly everything in this report: fal.ai (FLUX.2, LoRA training, BiRefNet, gpt-image-2 proxy) and either OpenAI direct or Ideogram (own transparent endpoints plus gpt-image-2.5 proxy). Replicate is the cheapest for bulk background removal but its per-second billing and cold starts make it a worse fit for latency-sensitive calls.

### Gaps

- Replicate's Node SDK docs, rate limits and its gpt-image listing were not fetched.
- Together's current image catalog and prices were not fetched; only blog mentions were found.
- fal's per-image prices for gpt-image-2, FLUX.2 flex/klein and Ideogram 4 are on individual model pages that were not fetched.

## Key question 6: Licensing, ownership, indemnification, platform disclosure

### Takeaway

Ideogram and Recraft (paid plan) disclaim ownership of outputs and permit commercial use; OpenAI assigns its rights in output to the user and offers Copyright Shield to API customers with exclusions; Adobe trains Firefly on Adobe Stock, public domain and openly licensed content and sells indemnification as an entitlement whose API coverage is unconfirmed; FLUX klein-4B weights are Apache 2.0 while [dev] and klein-9B are non-commercial. US law as of 2026: prompts alone do not make a user the author (Copyright Office, Jan 29, 2025), and the Supreme Court declined Thaler on March 2, 2026, leaving the human-authorship rule intact. Steam (rewritten Jan 2026) and itch.io (Nov 2024) both require disclosure of generative AI content shipped to players.

### Cited Findings

**Provider terms**

- OpenAI Terms of Use (published Jan 1, 2026): "We hereby assign to you all our right, title, and interest, if any, in and to Output"; output "may not be unique"; the Business Terms govern API use. — [OpenAI Terms of Use](https://openai.com/policies/terms-of-use/)
- Copyright Shield: OpenAI will "defend customers and pay the costs incurred" for copyright infringement claims, covering ChatGPT Enterprise and the API, not free/Plus. Exclusions in the API terms: customer knew or should have known output was infringing, safety/filtering features disabled or ignored, output modified or combined with non-OpenAI products, trademark claims from commercial use, Third Party Offering content. — [Proskauer](https://www.proskauer.com/blog/openais-copyright-shield-broadens-user-ip-indemnities-for-ai-created-content); [Classmethod (quotes Nov 2023 terms)](https://dev.classmethod.jp/articles/openai-devday-copyright-shield/)
- Ideogram: "We do not claim ownership of your generated images, and you're free to use them for any purpose, including commercial use." Price changes take effect on publication; material changes communicated 14 days ahead. — [Ideogram pricing](http://ideogram.ai/pricing/?pricing_tab=api)
- Recraft: paid-plan images "grant you full ownership and commercial rights" and stay private after cancellation; free-plan images are public, owned by Recraft, not licensed for commercial use (the pricing FAQ says "with limitations", an inconsistency); regardless of plan, outputs may not be used to train AI models. — [Recraft ownership docs](https://www.recraft.ai/docs/plans-and-billing/commercial-rights-and-ownership.md); [Recraft ownership FAQ](https://recraft.ai/blog/ownership-and-commercial-use-faq)
- BFL: FLUX.2 [klein] 4B weights Apache 2.0 "free for commercial use"; 9B under FLUX Non-Commercial license; [dev] "Open weights, non-commercial". — [BFL help](https://help.bfl.ai/articles/7424166364-wip-welcome-to-the-forest); [BFL pricing](https://docs.eu.bfl.ai/quick_start/pricing)
- Adobe: Firefly models "trained on licensed content, such as Adobe Stock, and public domain content where the copyright has expired" (earlier: also openly licensed content). IP indemnification is an entitlement "some customers will have the opportunity to purchase". Covered features listed in the product description include Text to Image, Generative Fill, Generative Expand. A February 2026 community question on whether API-generated images are indemnified got no authoritative answer. — [Adobe Firefly enterprise legal FAQ (2024-06-11)](https://adobe.com/cc-shared/assets/pdf/enterprise/firefly-legal-faqs-enterprise-customers-2024-06-11.pdf); [Adobe Firefly product description](https://helpx.adobe.com/legal/product-descriptions/adobe-firefly.html); [Adobe community, Feb 2026](https://community.adobe.com/questions-404/firefly-api-copyright-indemnification-and-c2pa-metadata-for-commercial-use-1549088)
- Leonardo free-tier output carries a non-exclusive commercial license and is public. — [GameDev AI Hub](https://gamedevaihub.com/best-ai-pixel-art-generators-for-2d-indie-games/) (secondary)
- PixelLab permits commercial use on every plan and restricts training other models; Retro Diffusion's FAQ permits personal and commercial use. — [GameDev AI Hub](https://gamedevaihub.com/best-ai-pixel-art-generators-for-2d-indie-games/) (secondary; for comparison only)

**US copyright**

- Copyright Office, "Copyright and Artificial Intelligence, Part 2: Copyrightability", January 29, 2025: "prompts alone do not provide sufficient human control to make users of an AI system the authors of the output"; using AI tools does not by itself deprive a work of protection, and human creativity "expressed through the use of AI systems ... continues to enjoy protection"; sufficiently detailed prompts may themselves be copyrightable. — [Sidley](https://datamatters.sidley.com/2025/02/07/u-s-copyright-office-issues-report-on-artificial-intelligence-and-copyrightability/); [IPWatchdog](https://ipwatchdog.com/2025/01/29/part-two-copyright-office-ai-report-says-creative-prompting-doesnt-constitute-authorship/); [Lowenstein](https://www.lowenstein.com/news-insights/publications/client-alerts/us-copyright-office-issues-report-on-copyrightability-of-generative-ai-outputs-ai)
- Thaler v. Perlmutter: D.C. Circuit held March 18, 2025 that the Copyright Act requires a human author; cert petition filed October 9, 2025 (No. 25-449); Supreme Court denied certiorari March 2, 2026 without opinion. Office policy requiring disclosure of AI-generated portions stands. — [Baker Donelson](https://www.bakerdonelson.com/supreme-court-denies-certiorari-in-thaler-v-perlmutter-ai-cannot-be-an-author-under-the-copyright-act); [Reed Smith](https://www.reedsmith.com/our-insights/blogs/viewpoints/102mlpl/supreme-court-denies-certiorari-in-thaler-v-perlmutter-human-only-rule-for-ai/); [Jones Day, Mar 2026](https://www.jonesday.com/en/insights/2026/03/jones-day-talks-women-in-ip--supreme-court-denies-review-of-thaler-v-perlmutter-ai-authorship-questions-remain)
- Background: the March 16, 2023 registration guidance and the Zarya of the Dawn decision (text and arrangement registered, Midjourney images not). — [Copyright Office guidance PDF](https://www.Copyright.Gov/ai/ai_policy_guidance.pdf); [Garrigues](https://www.garrigues.com/en_GB/garrigues-digital/us-copyright-office-publishes-new-guidelines-registration-ai-generated-works) (older)

**Steam**

- Steamworks Content Survey, third section: if AI was used during development or is in the product, describe it in detail; distinguishes Pre-Generated content (evaluated in prerelease review like any other content; confirm no illegal/infringing content) from Live-Generated (also describe guardrails; live-generated Adult Only sexual content not allowed); focus is "AI in creating content that ships with the game and is consumed by players", not efficiency tools. Some answers cannot be edited after approval without Steam Support. — [Steamworks Content Survey](https://partner.steamgames.com/doc/gettingstarted/contentsurvey)
- Valve "significantly" rewrote the rules on January 16–17, 2026: AI dev tools such as code helpers need no disclosure; two disclosure types remain, "AI to generate content for the game" (in-game, store page, marketing) and "AI content generated during gameplay"; disclosures appear on the store page under "AI Generated Content Disclosure" (required since 2024); roughly 8,000 titles disclosed in H1 2025. — [VGC, Jan 2026](https://www.videogameschronicle.com/news/valve-has-significantly-rewritten-steams-rules-for-how-developers-much-disclose-ai-use)
- A blog claims a stricter per-asset labelling rule took effect June 2026; this does not match Valve's page and is unverified. — [remio.ai](https://www.remio.ai/post/indie-game-studios-are-rejecting-steam-s-new-mandatory-ai-metadata-tags) (unreliable)

**itch.io**

- Quality guidelines: "accurately tag your project if it contains materials produced by generative AI" via the "AI Disclosure" section on the project edit page; tagged projects appear on the AI Assisted browse page; itch.io is "strictly enforcing disclosure for all game asset pages" and failing to tag an asset page "may result in delisting"; self-contained algorithms (pathfinding, procedural levels, dynamic music) are exempt; mass-produced AI pages may be treated as spam. — [itch.io quality guidelines](https://itch.io/docs/creators/quality-guidelines)
- The requirement was announced around November 21–22, 2024; untagged generative-AI assets lose browse indexing after a grace period, with user reports for enforcement. — [GamingOnLinux](https://gamingonlinux.com/2024/11/itchio-store-now-requires-ai-generated-content-disclosures-for-assets); [80.lv](https://80.lv/articles/asset-creators-on-itch-io-now-have-to-disclose-the-use-of-generative-ai/)

### Inferences

- For a shipped game, the registrable copyright will rest in the human-authored selection, arrangement and edits of sprites, not in raw generations; a pipeline that records prompts, references and manual edits per asset makes the Steam disclosure and any registration straightforward.
- Among native-alpha providers, Ideogram has the simplest written grant; OpenAI adds an indemnity but only when outputs are unmodified and filters are on, which most sprite post-processing (defringe, quantize) arguably breaks.

### Gaps

- OpenAI's Business Terms page (API ownership and indemnity text) returned 403; the ownership quote is from the consumer Terms of Use and the exclusions from a 2023 blog reproduction.
- BFL's API output ownership terms were not retrieved (only model-weight licenses).
- Whether Firefly Services API output is indemnified is unresolved even on Adobe's own forum.
- Shutterstock and Getty generative APIs were not researched within the call budget.

## Key question 7: Practitioner reports 2025–2026

### Takeaway

Nearly every 2026 write-up on AI sprites is vendor-authored; the recurring, consistent claims are: generate large and downsample rather than at sprite size; put several props on one sheet and cut them apart to cut cost ~6x; expect halos even from native alpha; pass an approved sprite as a reference on every later call; tilesets and animation frames are where generation breaks down. No independent postmortem comparing gpt-image and FLUX on sprites was found.

### Cited Findings

- Rangy (Aug 14, 2026; vendor): a six-prop sheet at $0.05 works out to ~$0.008 per asset vs ~$0.30 for six individual generations; a usable 40-asset set costs roughly a dollar after discards; "AI cannot produce true pixel art" (irregular grids, anti-aliased edges, too many colours), so downsample nearest-neighbour, quantize, hand-clean, and treat output as a redraw reference; animation sheets unreliable; painted/illustrated/vector-adjacent styles suit generation; recommends GPT Image 2 for sheets and the cheapest model that draws clean shapes. — [Rangy](https://rangy.ai/blog/ai-game-assets)
- OpenAI forum users (Aug 2026) building transparent assets with gpt-image-2: alpha 252–254 in opaque regions, grey/white halos, shadow with alpha; GIF output degraded quality so one user switched to animated WebP. — [OpenAI community thread](https://community.openai.com/t/transparent-backgrounds-are-now-available-in-preview-for-gpt-image-2-in-the-api/1391541)
- AgentBrush (vendor, 2026): gpt-image-2 with presets `pixel_art`, `isometric`, `flat_illustration`, `realistic`; local background removal after generation; reference sprite on later calls; token costs at 1024x1024 low 1 / medium 5 / high 20 tokens with $0.04 overage per token; tile seams need cropping; palette locking unreliable; sub-16x16 sprites look like downscaled illustration. — [AgentBrush](https://agentbrush.dev/blog/2d-game-asset-generation)
- Summer Engine (June 6, 2026; vendor, no named models or methodology): tilesets weakest ("visible seams"), animation drift, UI icons need cleanup at game size; use AI for static assets, handle animation separately, build tilesets deliberately on a grid. — [Summer Engine](https://www.summerengine.com/blog/ai-2d-game-asset-generator)
- NUILab handbook (updated Aug 9, 2026; GPU workflow): FLUX.2 klein-4B fp8 + Qwen3-4B encoder + pixel-art LoRA in ComfyUI; batches of ~8, cull aggressively; "don't generate at the target sprite size" ("diffusion models produce mush at 32x32"); the LoRA outputs transparent backgrounds directly; specify the palette early; keep prompts short; avoid in-image text; Aseprite for palette/cleanup/animation. — [NUILab](https://aigaming.nuilab.org/workflows/sprites/)
- Scenario blog (vendor): consistency across 80+ poses is the hard part; train a character model on 5–15 references. — [Scenario](https://www.scenario.com/blog/ai-sprite-generator)
- Spriteflow.io review (July 2026): 3–5 reference images or a preset; walk cycle ~70 s; output "may not match the polish" of professional work; hero assets touched up in Aseprite; exports to Unity/Godot/Unreal atlases. — [PC Tech Magazine](https://pctechmag.com/2026/07/cutting-sprite-production-time-from-weeks-to-minutes-without-losing-consistency/)
- GameDev AI Hub (reviewed Sept 19, 2026; affiliate links): no same-prompt tests; checklist: "Check the alpha channel", "A checkerboard baked into the image is not a transparent background", inspect pixel grids at native and zoomed size, look for stray semi-transparent pixels. — [GameDev AI Hub](https://gamedevaihub.com/best-ai-pixel-art-generators-for-2d-indie-games/)
- Recraft users report edge halos on generated transparent PNGs despite prompting against them. — [Recraft feedback](https://feedback.recraft.ai/bug-reports)
- SpriGen (GitHub, 2023): rembg for transparency, tiled textures, animated sprites "in progress". — [SpriGen](https://github.com/hafiidz/sprigen) (stale)
- Pixel-art-specific tools for comparison: Retro Diffusion offers reference-based generation; PixelLab's pitch is keeping "your character recognizable" across movements; pixie.haus animates from a starting image. — [GameDev AI Hub](https://gamedevaihub.com/best-ai-pixel-art-generators-for-2d-indie-games/)

### Inferences

- The practitioner consensus maps onto a concrete Node pipeline: (1) generate at 1024+ with a native-alpha model or on a flat key colour; (2) several props per sheet when the style tolerates it; (3) normalize alpha, defringe, erode 1 px; (4) nearest-neighbour downsample to the sprite grid and quantize to a fixed palette; (5) keep the approved base sprite as a reference input for every related call; (6) do tiles and animation with a tile-aware tool or by hand.

### Gaps

- No first-person indie postmortem comparing providers on the same sprite brief was found; all 2026 sources with concrete numbers sell a tool.
- No quantitative report on how many of N generations with a reference image stayed on-model.
