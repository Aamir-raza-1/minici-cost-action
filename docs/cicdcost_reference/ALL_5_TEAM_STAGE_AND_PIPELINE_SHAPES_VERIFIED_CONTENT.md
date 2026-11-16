# Extracted Reference Content: 5 Team-Stage & Pipeline-Shape Models


---
# PAGE: `ci-cd-cost-for-startups`
Home / Per team stage / Startups 
## CI/CD cost for startups in 2026: monthly bills at 1-15 dev scale

Most startups overpay for CI early and underinvest in it later. The free tiers in 2026 are genuinely generous, particularly GitHub Actions, and cover the entire CI need for the first 6-12 months of a typical SaaS startup. The right move is usually to spend the early months on the product, not on optimising a $30 bill, and to upgrade only when concrete signals demand it. This page maps the realistic monthly bill at each team-size band, the upgrade triggers worth watching, and the early traps to avoid.
 
Headline at a glance (2026)

Solo to 5 devs: $0-50/month. 5-10 devs: $50-200/month. 10-15 devs: $150-500/month. Add 1.5-3x if you ship iOS. Add nothing meaningful for security-scanning add-ons (most are free at startup scale). Most startups optimise this line item too early; the bigger wins are at 25-100 dev scale on the scaleup page .
 
### Solo founder / 1-2 devs

The free tiers cover essentially every reasonable workload. GitHub Actions Free includes 2,000 Linux minutes per month per personal account. A solo founder typically pushes 30-50 times per week, with each CI run consuming 4-8 minutes. Monthly burn: 600-1,600 minutes. Comfortably inside the free tier with headroom for the inevitable bad week.

The only structural cost at this scale is the GitHub seat itself, and even that is free at the personal-account or small-team level. The few exceptions: a private repo on GitHub Free does have lower included minutes than a Team plan; if you are already paying for a team account, the included minutes scale with seats. The actionable advice: do not optimise. Spend the engineering time on the product.

### Pre-seed / 3-5 devs

Still mostly free, with occasional overage in busy weeks. At 3-5 devs pushing 5 times each per workday and 6-minute CI runs, monthly burn is roughly 1,800-3,000 minutes. The GitHub Actions Free 2,000 minutes is borderline; a typical pre-seed team has 2-3 months of the year where they nudge over the limit and pay $5-15 in overage. Annualised cost: $50-150 if you stay on Free, $100-200 if you upgrade to Team for the cleaner allowance plus the team-management features.

The upgrade to Team becomes worth it when you have more than two engineers reviewing PRs and you start wanting protected branches, CODEOWNERS files, and required status checks. Those features come standard with Team plan ($4/user/month). For a 4-person team, $16/month for the operational improvements is a reasonable trade.

CircleCI at this scale is more workable than it used to be: the Free plan now includes 30,000 credits/month (about 3,000 Medium Linux minutes) and 30 concurrent jobs for up to 5 active users. The constraint is that Free has no overage, so a busy month simply stops your builds. The Performance upgrade (from $15/month, 5 active users included) unlocks credit overage, but you have moved out of the "CI is essentially free" band that GitHub Actions provides.

### Seed / 6-10 devs

You have crossed onto paid CI for most platforms. GitHub Actions Team at $4/user x 10 = $40/month for seats, plus 3,000 included minutes (often consumed by week three of the month), plus overage at $0.006/min. Monthly burn at this team size is typically 8,000-15,000 minutes. Net bill: $40 seats + 5,000-12,000 minutes x $0.006 = $70-112 in compute, plus seats, total roughly $110-150/month.

This is the band where simple cost-control moves start paying back. Caching dependencies (npm, pip, Maven) cuts cold-install time 30-60% and roughly 15-25% of the bill (see build cache savings ). Path filtering on a multi-service repo cuts another 30-50% (see branch protection cost ). Two days of platform-engineering time can take the monthly bill from $150 to $80; two days at $1,500 fully loaded is $3,000, payback in 3-4 years. Not high ROI yet, but the patterns you set now scale.

### Series A / 10-15 devs

Now meaningful CI spend. 12-15 devs at 5 pushes/dev/day generates 18,000-25,000 minutes per month. GitHub Actions Team at $4/user x 15 = $60/month for seats, plus 25,000 - 3,000 = 22,000 minutes overage at $0.006 = $132. Total $192/month for compute. Add macOS minutes if you ship iOS: a 5-engineer iOS sub-team doing 8 builds each per day at 12 minutes each is 9,600 macOS minutes, billable at $0.062 = $595/month for iOS alone. Total bill: $192 + $595 = $787/month, with iOS dominating.

This is where the right team starts thinking about cost optimisation as a project. The first investment is usually a Mac mini for iOS CI (one-time $700, electricity $5/month, replaces $500+/month in hosted macOS). The second is dependency caching across the whole stack. The third is path filtering on the multi-service backend. Combined, these typically take a $790/month bill to $250-350/month.

### The early traps

Three traps startups consistently fall into. First, premature self-hosting. Setting up self-hosted runners on AWS at 8 developers because "it will be cheaper" consumes a week of engineering time to save $50/month. The maths does not work below 25 devs. Defer.

Second, premature multi-platform CI. A startup with 3 developers running their suite on Linux, Windows, and macOS for "portability" is paying 13x the bill (1x + 2x + 10x OS multipliers) to cover platforms nobody will use until enterprise customers demand them. Run multi-platform on a nightly cron, run Linux on every PR.

Third, ignoring the macOS line for too long. iOS CI on hosted macOS at any non-trivial volume becomes the dominant cost line within months. The Mac mini intervention is the highest-ROI infrastructure investment a small iOS team can make. Do it as soon as you have a second iOS engineer on the team.

### When to graduate to scaleup planning

The signals that you have outgrown startup-mode CI thinking: monthly bill above $500, total devs above 15, concurrent-job cap hitting more than once a week, your CI bill is now a meaningful percentage of total infra spend. At that point the right move is to plan for the next two years of scale, not just optimise the next month. The patterns that work at 25-100 devs are different from the ones that work at 5-15. We cover those on the scaleup CI cost page .

For the immediate vendor-by-vendor monthly total at your specific build profile, plug numbers into cicdcalculator.com . This page is the cost-shape reference; the calculator gives the dollar number.

### Frequently Asked Questions
 How much does CI/CD cost for a startup? 
Solo founder or 1-2 dev pre-seed startup: $0/month, free tiers cover everything. 3-5 dev seed-stage: $0-50/month, mostly still on free tiers with occasional overage. 6-10 dev seed-to-Series-A: $50-200/month, you have crossed onto paid plans for most platforms. 10-15 devs Series-A: $150-500/month depending on language stack and build frequency. iOS development bumps these numbers 1.5-3x because of macOS runner pricing.
 Which CI/CD platform is cheapest for startups? 
GitHub Actions for any team using GitHub: 2,000 free Linux minutes/month on Free plan, 3,000 on Team. For OSS projects, GitHub Actions is free at any scale on public repos. CircleCI Free is workable for small teams (30,000 credits/month, 30 concurrent jobs) but offers no overage, so growing teams move to Performance (from $15/mo, 5 active users included). GitLab CI Free includes 400 minutes which is tight; teams committing to GitLab usually pay $29/user for Premium. For most US/EU startups in 2026, GitHub Actions plus the Team plan is the default.
 When should a startup upgrade from free CI? 
Three signals: free-tier minute consumption is consistently above 80% (you will hit overage soon), concurrent-job cap is causing PR queueing more than once a week, or you need a feature only paid plans provide (SAML, audit logs, additional protected branches). Most startups upgrade between months 6 and 12 of life. Avoid upgrading earlier; the free tier is generous and the engineering time spent setting up cost optimisations is better spent on the product.
 Should a startup self-host CI runners? 
No, almost always. Self-hosting saves money only above ~25,000 build minutes/month, which most startups do not hit until 25-50 developers. Below that threshold, the engineering time to set up and operate self-hosted runners is worth more than the compute saving. The only exception: heavy iOS workloads where a $700 Mac mini in the office pays back at any team size, because hosted macOS pricing is severe.
 What about AWS CodeBuild for an AWS-shop startup? 
Reasonable choice if your stack is AWS-native. CodeBuild costs $0.005/min for the smallest Linux compute class, slightly cheaper than GitHub Actions Linux. Free tier is 100 minutes/month which is too small to be useful. The integration story (CodePipeline, CodeDeploy, ECR) is good if your team is committed to AWS native tooling. The cost difference vs GitHub Actions at startup scale is small; pick based on integration fit, not price.
 
### Related stages and components
 
- CI/CD cost for scaleups 
- Enterprise CI/CD cost 
- Build minutes cost across vendors 
- CI runner cost: hosted vs self-hosted 
- Build cache savings 
- Cheapest CI by team size

---
# PAGE: `ci-cd-cost-for-scaleups`
Home / Per team stage / Scaleups 
## CI/CD cost for scaleups in 2026: the bill that doubles every quarter

The 25-100 developer band is where CI cost stops being a rounding error and starts being a finance line item. Headcount grows, codebases compound, and required-check sets accumulate faster than anyone removes them. Bills that were $300/month at 15 devs become $3,000/month at 50, often inside a year. This page maps the realistic bill at each band, the tier-upgrade decisions worth deferring or accelerating, and the structural moves that bring spend back under control without losing safety.
 
Headline at a glance (2026)

25 devs: $500-1,500/month. 50 devs: $1,500-5,000/month. 100 devs: $4,000-12,000/month. iOS-heavy teams add 30-80%. Teams with caching, path filtering, and a quarterly required-check audit land at the lower end of these ranges. Teams without these structural moves land at the upper end and grow faster than headcount.
 
### 25 devs: the optimisation tipping point

At 25 devs, monthly compute spend is typically $500-1,500. Roughly 30,000-60,000 build minutes per month depending on push frequency and run duration. GitHub Actions Team at $4/user x 25 = $100 in seats, plus $300-600 in Linux compute, plus $100-800 in macOS for iOS-heavy teams. Storage and artifacts add another $50-150.

This is the band where structural cost-control moves earn back their investment in weeks, not months. The two highest-ROI projects: dependency caching across all repos (1-2 days of work, 20-30% saving on the bill, payback in 2-3 months) and path filtering on monorepos or multi-service repos (1 week of work, 30-50% saving, payback in 3-6 weeks). Both pay back so fast that deferring them past quarter is cost-leadership malpractice.

The tier-upgrade pressure at 25 devs is mostly compliance-driven: SAML SSO requirement from your first enterprise customer, audit log requirements for a SOC 2 audit. These trigger a Team-to-Enterprise upgrade ($4 to $21 per user), which is +$425/month for 25 users. Defer until the customer requirement is real and signed.

### 50 devs: the structural decisions land

At 50 devs, monthly compute is typically $1,500-5,000 with a wide range driven by stack choice and discipline. The 60 concurrent job cap on GitHub Actions Team starts hitting weekly during busy hours. Teams either upgrade to Enterprise (+$850/month for 50 users) or move to self-hosted runners (one-time week of work, ongoing $300-700/month operational cost for the runner pool).

Self-hosted is the structurally better choice for teams in this band. The operational cost is comparable to the Enterprise upgrade, but you also escape the concurrent-job ceiling and gain control over runner sizing (bigger machines for the long-tail slow tests). The risk is that you now own a runner controller; if your platform team is one engineer, this is real exposure on a single point of failure. The risk-mitigated path: hosted runners for the on-call critical path (deploy pipelines), self-hosted for the bulk PR check workload.

Worked example: a 50-dev SaaS team with a Node backend monorepo (5 services) and a small iOS app (3 engineers). Monthly burn: 80,000 Linux minutes, 7,500 macOS minutes. On GitHub Actions Team: $200 seats + 80,000 x $0.006 = $480 Linux + 7,500 x $0.062 = $465 macOS = $1,145/month. After dependency caching (-25% on Linux, -10% on macOS) and a Mac mini for iOS in the office (-90% on macOS): $200 seats + $360 Linux + $47 macOS = $607/month. Saving $538/month for 1 week of platform engineering plus a $700 hardware purchase, payback in 6 weeks.

### 75 devs: the platform-engineering pivot

At 75 devs, the team needs at least one full-time platform engineer or DevEx specialist. The CI bill alone justifies it: $3,000-8,000/month in compute with 30-50% optimisation potential equals $1,000-4,000/month savings, before counting the developer-time cost of slow CI. A platform engineer costing $200k fully loaded ($16,500/month) breaks even on CI cost reduction alone for the savings ranges above; the deployment-frequency and reliability benefits are upside.

The platform investment shifts the CI cost curve from linear to sub-linear in headcount. Teams that hire platform early (around 50-75 devs) keep CI spend per developer roughly flat as they grow. Teams that defer the hire watch CI spend per developer grow as the codebase complexity outpaces the per-dev efficiency.

### 100 devs: scale economics take over

At 100 devs, you are running a small-medium engineering organisation. Monthly compute is typically $4,000-12,000. The structural moves that mattered at 50 devs are table stakes by now: caching, path filtering, self-hosted runners, affected-only test selection on monorepos. The new pressures are concurrency at scale (you need a runner pool of 20-50 machines), security and compliance overhead (SOC 2 audit, dedicated audit logs, fine-grained permissions), and per-team cost attribution (which team is driving the largest portion of the bill).

The cost-attribution work at this scale is non-trivial and worth doing. Tagging CI runs by team, owner, or service lets the platform team show finance which teams are overspending. A dashboard that shows monthly CI cost per service drives the right conversations: a 6-engineer team consuming 40% of CI compute usually has a fixable inefficiency they did not realise existed. The visibility alone often delivers 15-25% reduction without requiring a top-down efficiency mandate.

### The required-check audit pattern

Required-check sets accumulate. Each new check seemed like a good idea at the time. Most have low signal value: they pass 99% of the time and the rare failure is a flake, not a real bug. Quarterly review with the test: in the last 90 days, how many real bugs has each required check caught? Demote anything below 1, delete anything below 1 for two consecutive quarters.

A typical scaleup audit finds 30-40% of required checks are dead weight. Removing them does not reduce safety meaningfully (real bugs are still caught by the high-signal subset) and cuts the CI bill proportionally. We cover the audit framework in detail on the branch protection cost page .

### When to consider a CI vendor migration

Migrations are expensive: 2-6 weeks of platform engineering, plus the inevitable workflow translation issues. Worth it only when the destination is structurally better than the origin. Common migration patterns at scaleup: GitLab to GitHub Actions (because GitLab Premium per-seat is steep), Jenkins to GitHub Actions or GitLab (operational burden of Jenkins exceeds the per-minute saving), CircleCI to GitHub Actions (concurrent-job pricing model favours GitHub at high seat counts).

For per-vendor monthly totals at your specific build profile to inform the migration decision, plug numbers into cicdcalculator.com . The calculator is the right tool for vendor side-by-side; this page is the cost-shape reference for understanding why the numbers come out where they do.

### Frequently Asked Questions
 What does CI/CD cost at 50 developers? 
A 50-developer team with mixed Linux + iOS workloads on GitHub Actions Team typically pays $2,500-5,000/month in 2026: $200 in seats, $1,500-2,500 in Linux compute, $800-2,500 in macOS compute, and $50-200 in artifact + log storage. The macOS line dominates for any iOS-heavy team. The compute bill scales near-linearly with developers if no optimisations are in place; teams with caching and path filtering land 30-50% lower.
 Why does the CI bill double every quarter at scaleup stage? 
Three compounding pressures. First, headcount grows linearly. Second, codebase grows super-linearly: more services, more tests, longer builds. Third, the team adds new required-status checks (security scans, type checks, integration tests) faster than it removes them. Combined, the bill grows roughly as headcount times codebase complexity. The structural fix is path filtering, affected-only test selection, and a quarterly required-check audit.
 When should a scaleup move to self-hosted runners? 
When monthly compute spend exceeds $500-800, the savings from self-hosted runners on EC2 spot instances start covering the operational cost of running the runner pool. Below that, the labour cost of an Actions Runner Controller cluster on Kubernetes is more than the compute saving. Above $1,500/month in compute, self-hosted is almost always worth the investment. The transition typically takes 1-2 weeks of platform engineering work.
 Should scaleups upgrade to GitHub Enterprise? 
Only when concrete features force it: SAML SSO requirement (often customer- or compliance-driven), advanced audit logs, or hitting the 60 concurrent job cap on Team. The price jump from Team ($4/user) to Enterprise ($21/user) is significant; for a 50-dev team that is +$850/month. Defer until the requirements are real, not anticipated. SAML is the most common forcing function.
 What is the biggest waste in scaleup CI bills? 
Required-check duplication. Most scaleup repos run the same lint check, the same unit test suite, and the same integration tests on every PR push regardless of what changed. Adding path filters and affected-only test selection routinely cuts CI minutes 50-70%. The work is one focused engineering project (1-2 weeks) and the payback is measured in weeks. Surprisingly few teams do it because it competes with feature work in sprint planning.
 
### Related stages and components
 
- CI/CD cost for startups 
- Enterprise CI/CD cost 
- CI runner cost: hosted vs self-hosted 
- Parallelism cost 
- Branch protection cost 
- Build cache savings

---
# PAGE: `ci-cd-cost-for-enterprise`
Home / Per team stage / Enterprise 
## Enterprise CI/CD cost in 2026: where the line items multiply

Enterprise CI cost is structurally different from scaleup. The per-minute compute rate is similar; what changes is the licence step (Enterprise tiers cost 2-5x the Team-tier per-seat), the compliance overhead (audit logs, SAML/SCIM, dedicated tenancy), and the regional sprawl (CI in multiple regions for data residency). Bills run $25k-$500k/month at the 250-2,000 dev band, with compute often a minority of the total. This page maps the realistic spend, the line items that drive enterprise economics, and the procurement decisions that make the difference between a $40k/month bill and a $150k/month bill at the same team size.
 
Headline at a glance (2026)

250 devs: $25k-$80k/month. 500 devs: $50k-$150k/month. 1,000+ devs: $100k-$500k+/month. Compute is typically 30-50% of the bill at this scale; seats, security add-ons, and dedicated tenancy make up the rest. FedRAMP and other regulated-industry tiers add 1.5-3x on the per-seat baseline. The optimisation focus shifts from compute efficiency to procurement strategy.
 
### The enterprise per-seat step

GitHub Enterprise jumps from Team's $4/user to $21/user, a 5.25x increase. GitLab Premium-to-Ultimate goes from $29/user (billed annually) to custom pricing negotiated with sales. CircleCI Performance-to-Scale is quoted but typically 2-3x. Each step purchases a bundle: SAML SSO, SCIM provisioning, advanced audit logs, deployment environments with required reviewers, IP allow-listing, custom organisational roles, and the relevant security scanning add-on (GitHub Advanced Security, GitLab Ultimate's security suite).

The bundle is not negotiable per-feature. You cannot buy "just SAML" on GitHub at the Team price. The forcing functions for the upgrade are usually customer-driven: an enterprise customer or a SOC 2 auditor mandates SAML SSO or audit log retention, and the upgrade follows. Plan procurement around when the forcing function will land, not at the day before the auditor's site visit.

Multi-year deals can pull list price down by 15-25% at the enterprise tier. Vendors prefer multi-year for revenue predictability. The trade-off is reduced flexibility if you decide to migrate vendors mid-term. For organisations with stable engineering teams and committed CI vendor choices, the multi-year discount is meaningful at this scale; for fast-moving organisations the flexibility cost may exceed the discount.

### Compliance overhead as a line item

SOC 2 Type II compliance requires audit log retention typically 1+ years, often 7 years for financial-services scope. Vendor-default audit log retention is 90-180 days; longer retention either requires the next pricing tier (typical) or shipping logs to your own SIEM (always available). Datadog Logs at 1-year retention for the kind of audit-log volume an enterprise generates is typically $5k-$25k/month. Splunk for the same scope is similar. We covered the per-GB storage maths on the log retention cost page ; at enterprise scale the multiplication is severe.

ISO 27001 adds documentation overhead (policies, procedures, risk assessments) that is engineering time more than vendor cost, but the CI vendor must support the relevant controls. PCI DSS for financial-services workloads requires segregation of duties between deployment-permission holders and code-merge-permission holders, which often forces the deployment-environments feature on GitHub Enterprise or the equivalent on GitLab. HIPAA for healthcare PHI workloads requires Business Associate Agreements (BAAs) which most CI vendors only sign at Enterprise tier and above.

### Regional residency and the geo-multiplier

Multinational organisations frequently need CI workloads pinned to specific regions for data-residency reasons. GDPR requires EU-region data residency for EU customer data. Data sovereignty laws in Australia, Canada, and increasingly several US states impose similar regional pinning. The mechanics: a CI runner in eu-west-2 cannot ship logs to a US-region observability backend without breaching residency.

The cost: each region is roughly a parallel infrastructure deployment. A 500-dev organisation operating in US, EU, and APAC regions pays roughly 2.5-3x the single-region equivalent for runner pools, log storage, and artifact storage. The savings from cross-region consolidation are unavailable. We covered the cross-region egress dimension on the cross-region egress cost page , and the same maths applies at scale.

### Dedicated tenancy and the security premium

For organisations whose security policy mandates isolation from other tenants, vendors offer dedicated tenancy at substantial premium. GitHub Enterprise Cloud with a dedicated tenancy add-on, GitLab Dedicated, CircleCI Server. Premiums typically run 40-100% on top of the standard enterprise rate.

The forcing functions for dedicated tenancy: regulated industry obligations (defence, financial services, healthcare), large enterprise security policies that explicitly forbid multi-tenant SaaS for source-control-adjacent systems, or contractual obligations to specific government customers. The decision is not usually optional; if your customer demands it, you pay it. The cost-management angle is to negotiate the premium aggressively (multi-year commits, volume discounts) and to be precise about which workloads actually need the isolation versus which can stay multi-tenant.

### FedRAMP and government tiers

For US federal customers, the CI environment must run in a FedRAMP-authorised region. GitHub Enterprise Cloud has a FedRAMP Moderate authorisation. GitLab offers GitLab Dedicated for Government. CircleCI offers a Server option that customers can run in their own FedRAMP-authorised cloud. Per-seat pricing for FedRAMP-tier environments is quoted rather than list-priced and typically lands 1.5-3x the standard enterprise rate, with annual minimum commitments.

Other government tiers (UK G-Cloud, Australian IRAP, Canadian PBMM) follow similar patterns: a separate authorised environment, separate pricing, separate procurement. Plan for a multi-month sales cycle plus a multi-month onboarding to a new vendor environment for any of these tiers.

### Where compute optimisation still matters at enterprise scale

Compute is typically 30-50% of the enterprise CI bill. At 250 devs and a $50k/month total, that is $15-25k/month in compute, which is still worth optimising. The structural moves (caching, path filtering, affected-only test selection, self-hosted runner pools) all apply and the savings are larger in absolute terms than at scaleup scale. A 25% reduction on $20k/month is $5k/month, $60k/year, which justifies sustained platform-engineering investment.

The discipline shifts from project-based optimisation to programme-based: a permanent platform-engineering function that owns CI cost as a KPI, with a per-team or per-service cost-attribution dashboard, and a quarterly required-check audit. The biggest wins at enterprise scale are not technical; they are organisational. Teams that overspend usually do so because nobody at finance or platform-leadership is paying attention.

### Frequently Asked Questions
 What does CI/CD cost at 250 developers? 
Enterprise CI/CD at 250 developers typically runs $25,000-$80,000/month all-in. GitHub Enterprise at $21/user x 250 = $5,250 in seats. Compute averages $10,000-30,000/month depending on stack and disciplines. Add $5,000-15,000 for security scanning add-ons, audit log retention, and observability. Add $5,000-30,000 if you require dedicated tenancy or regional residency. The compute portion is usually well-controlled at this scale; the licence and compliance portions dominate.
 Why does the per-seat price double for enterprise? 
Enterprise per-seat prices include features that smaller teams do not need: SAML/SCIM provisioning, advanced audit logs, GitHub Advanced Security or equivalent, dedicated support SLA, deployment environments with required reviewers, IP allow-listing, custom roles. These features are non-negotiable for organisations with SOC 2, ISO 27001, or regulated-industry obligations. The bundle is not negotiable per-feature, so the per-seat jumps in a step rather than scaling.
 What is dedicated tenancy in CI/CD pricing? 
Dedicated tenancy means your CI workloads run on infrastructure isolated from other vendor customers, rather than on shared multi-tenant compute. Vendors charge a substantial premium for this: typically 40-100% on top of the standard list price, often quoted rather than published. The use cases that demand it: regulated industries (financial services, defence, healthcare PHI), regional data residency requirements (EU GDPR with no US-region routing), and sometimes large enterprise security policies that mandate isolation.
 Should an enterprise self-host CI? 
Frequently yes for the runner pool, rarely for the orchestration layer. Self-hosted runners on a Kubernetes cluster operated by a platform-engineering team is the standard enterprise pattern: it provides cost control, security boundary control, and concurrency at scale. Self-hosting the orchestration layer (GitHub Enterprise Server vs GitHub Enterprise Cloud, GitLab self-managed vs GitLab.com) is a more nuanced decision driven by data residency, network topology, and the platform team's appetite for operating a critical-path service.
 What does FedRAMP CI cost? 
FedRAMP-authorised CI environments are quoted, not list-priced. The vendor needs to operate in a FedRAMP-certified region (GitHub Enterprise Cloud has a FedRAMP Moderate offering; GitLab and CircleCI offer dedicated environments). Per-seat pricing typically lands 1.5-3x the standard enterprise rate, plus annual minimums. For a 250-dev team operating under FedRAMP, expect annual contract values of $750k-$2M for CI alone, before counting the compliance overhead of operating to the standard.
 
### Related stages and components
 
- CI/CD cost for startups 
- CI/CD cost for scaleups 
- Enterprise CI/CD pricing comparison 
- Log retention cost 
- Cross-region egress cost 
- CI runner cost: hosted vs self-hosted

---
# PAGE: `monorepo-ci-cost`
Home / Per pipeline shape / Monorepo 
## Monorepo CI cost in 2026: where the minutes go in a unified codebase

Monorepos concentrate engineering work in a single repository where any change can theoretically affect any other code. Without selective test execution, every push runs the entire test suite, which scales the CI bill with codebase size rather than with change volume. The affected-only pattern (Nx, Turborepo, Bazel) brings the bill back to linear in change volume, where it belongs. This page covers the cost shape of monorepo CI in 2026, the tooling that makes it tractable, and the realistic before-and-after numbers from teams that adopted the pattern.
 
The change-detection problem

A naive monorepo CI runs every test on every push: a one-line PR to fix a typo runs the same 90-minute test suite as a major refactor. With Nx affected-only or Bazel selective testing, the same PR runs only the 12 tests that depend on the changed file. Result: 70-95% reduction in CI minutes per PR, with no loss of safety on the typical small change.
 
### Why monorepo CI cost scales super-linearly

Three compounding effects. First, the test suite grows roughly with codebase size. A 10-service monorepo has roughly 10x the tests of a single-service repo at similar maturity. Second, the PR rate grows roughly with developer count, and developer count tends to scale with codebase size. Third, the required-check set tends to grow with the diversity of services in the repo: each new language, framework, or integration adds checks that must run.

Compounded, a 50-developer monorepo with 15 services and 4 required checks per service generates 50 x 5 (pushes/dev/day) x 20 (working days) x 60 (4 checks x 15 services) checks per month = 300,000 check executions. At 4 minutes average per check that is 1.2 million minutes, $7,200 at $0.006/min Linux. The actual bill is usually a fraction of this because not every check runs against every service, but the worst-case shape is the unconstrained run-everything-on-every-push pattern.

The fix is structural: only run tests for services that are affected by the change. The dependency graph is the new infrastructure component you maintain.

### Affected-only with Nx

 Nx is the dominant tool for JavaScript and TypeScript monorepos. The model: you define a project structure (apps and libs), and Nx infers the dependency graph from imports. nx affected --target=test runs tests only for projects affected by the changes in the working tree (or, in CI, between two git refs).

Implementation effort: moderate. Existing JS monorepos can usually adopt Nx incrementally over 2-4 weeks. The dependency graph builds from imports automatically; the developer-facing change is wrapping common commands (test, build, lint) with the nx CLI. CI integration: replace npm test with nx affected --target=test --base=$BASE_SHA --head=$HEAD_SHA .

Realistic savings: 60-85% reduction in test minutes per PR for a typical mature monorepo. The variance comes from change profile: PRs that touch shared libraries trigger many affected projects, so the savings on those PRs are smaller. PRs that touch one specific service trigger a small affected set with large savings.

### Affected-only with Turborepo

 Turborepo is the lighter-weight alternative to Nx for JS/TS monorepos. The conceptual model is similar: define tasks per package, Turborepo computes the affected set from changes. The big draw is the simpler config and the integration with Vercel's remote cache product.

Implementation effort: lower than Nx for greenfield setups. Existing repos with established build tooling (Webpack, Vite, custom build scripts) sometimes need significant adaptation to fit the Turborepo task model. CI integration: turbo run test --filter=...$BASE_SHA .

Realistic savings: similar to Nx at 60-85% per PR. The remote cache (Vercel-hosted at $25-100/seat/month, or self-hosted at the cost of an S3 bucket) adds another layer: any task previously executed with the same inputs returns from cache rather than re-running.

### Bazel for the largest monorepos

For very large monorepos (250k+ lines, 50+ services, multi-language) Bazel is the canonical choice. Bazel's hermetic build model and content-addressed cache are designed precisely for the "run only what changed" problem at the largest scale. Google, Pinterest, Airbnb, and many similar engineering organisations run Bazel for this reason.

The migration cost is significant. Existing repos typically need 6-12 months of engineering time to fully adopt Bazel because every BUILD file must be authored explicitly and every dependency must be declared. The payoff is order-of-magnitude reduction in CI minutes for very large codebases, plus the operational benefits of hermetic builds (reproducibility, distributed caching, cross-language consistency).

Bazel is not the right answer for smaller monorepos: the migration cost exceeds the saving. The threshold where Bazel pays back is roughly the 250k-line, 50-service mark, where the lifetime CI savings exceed the multi-month migration. Below that, Nx or Turborepo deliver most of the benefit with much less migration cost.

### Worked example: 50-dev JS monorepo

A 50-developer JS monorepo with 15 services. Without affected-only: every PR runs all 15 service test suites at average 6 min each = 90 min per push. 50 devs x 5 pushes/day x 20 days x 90 min = 450,000 min/month. At GitHub Actions Linux $0.006: $2,700/month, plus the rounding tax (each of those 90-min jobs is one job, so rounding is small for individual jobs but the total is right).

With Nx affected-only: average PR touches 2-3 services. New per-PR cost: 3 services x 6 min = 18 min. New monthly: 50 x 5 x 20 x 18 = 90,000 min/month, $540 at GitHub Actions Linux. Saving: $2,160/month, or roughly 80% reduction. The Nx adoption project takes about a month for a team familiar with the tool, so payback is roughly 2-3 weeks of CI cost.

Add the Nx Cloud distributed cache on top: another 30-50% reduction because tasks previously executed with the same inputs return from cache. New monthly cost: $300-450. Total saving from no-tooling baseline: 85-90%.

### The dependency-graph maintenance cost

Affected-only is not free of operational cost. The dependency graph must be maintained: when a developer adds a new shared dependency between two services, the graph must update or the affected-only logic will miss it. With Nx and Turborepo this is automatic if developers use standard imports; with Bazel it is explicit in BUILD files.

The failure mode of dependency graph drift is silent: a test gets skipped that should have run, a regression escapes to production. The mitigation is a nightly "run everything" build that catches what affected-only missed during the day. The nightly run is more expensive per execution than a single PR but runs only once a day, so the total monthly cost is small.

### Frequently Asked Questions
 Why does monorepo CI cost more than multi-repo? 
Without affected-only test selection, a monorepo runs the entire test suite on every push regardless of what changed. A multi-repo setup only runs the test suite of the repo that changed. So a one-line change to a tiny utility runs the same 90-minute test suite as a major refactor of the platform. Monorepo CI cost scales with codebase size; multi-repo CI cost scales with change volume. Affected-only selection brings monorepo back to scaling with change volume, where it belongs.
 What is affected-only test selection? 
Affected-only test selection runs only the tests that depend on changed files. Tools build a dependency graph of the codebase: which files import which, which tests cover which production files. A change to file X triggers only the tests that transitively depend on X. Tools: Nx (JS/TS monorepos), Turborepo, Bazel (multi-language), Pants (multi-language). Realistic savings: 70-95% of test minutes on a typical monorepo PR.
 Is Bazel worth it for CI cost? 
Yes for codebases above ~250k lines or above ~20 services. Bazel's hermetic build model and remote cache deliver near-perfect cache reuse: any file or test that has been built or run before with the same inputs returns from cache instantly. The setup cost is significant: typically 6-12 months of engineering time to migrate a non-Bazel repo. The ongoing operational cost is lower than the alternatives once you are past the migration. Below the size thresholds, Nx or Turborepo deliver most of the benefit with much less migration cost.
 What is the typical CI bill for a monorepo? 
Highly variable. A 50-developer JavaScript monorepo with 15 services and no affected-only selection typically pays $3,000-8,000/month in CI. The same team with Turborepo and remote caching typically pays $800-2,000. The same team again with Nx Cloud and full distributed caching often lands at $400-1,200. The 5-10x range between worst and best is achievable with available tools; the gap is closed by engineering investment, not by switching CI vendors.
 Should every team use a monorepo? 
No. Monorepos work well for tightly-coupled product teams shipping a single application or platform. They work poorly for autonomous teams that want independent release cadences and tooling choices. The CI cost story is not a primary reason to choose either model: with affected-only selection, monorepo CI is competitive with multi-repo. With unrestricted full-suite runs, monorepo CI is dramatically more expensive.
 
### Related cost components and shapes
 
- Microservices CI cost 
- Branch protection cost 
- CI/CD caching strategies 
- Build cache savings 
- Parallelism cost 
- CI/CD cost for scaleups

---
# PAGE: `microservices-ci-cost`
Home / Per pipeline shape / Microservices 
## Microservices CI cost in 2026: when 50 repos cost 50x

Microservices spread CI across many repositories, each with its own pipeline. The simple multiplier is brutal: 50 services means 50 pipelines, each running on every push to each service. The per-service overhead alone (image pull, cache restore, framework setup) consumes 25-40% of compute. The structural fix is shared CI templates plus per-service caching plus a clear ownership model between platform and service teams. This page walks through the cost shape, the templates and ownership patterns that scale, and the realistic before-and-after numbers from teams that adopted them.
 
The N-services multiplier

50 services x 10 pushes/day each x 5-minute pipeline = 250 hours of compute per day, or 5,000 hours/month. At GitHub Actions Linux $0.006/min that is $1,800 in compute per month. Plus the per-service setup overhead. Plus per-service required-check accumulation. Total realistic microservices CI bill: $3-8k/month at this scale.
 
### Where the multiplier comes from

A microservices CI bill scales as services x pushes x average_pipeline_minutes. Each multiplier compounds. Services grow as the architecture matures (10 to 30 to 80 services over time). Pushes per service grow as the team grows (each service gets more contributors). Average pipeline minutes grow as test suites mature.

Worse, much of the per-pipeline time is overhead unrelated to the service's actual work. A typical microservices pipeline spends 30-90 seconds pulling base images, 30-60 seconds restoring caches, 15-30 seconds initialising the test framework. Total setup overhead: 60-180 seconds per pipeline run. With 500 pipeline runs per day, setup overhead alone consumes 8-25 hours of compute daily, or 240-750 hours per month.

The setup overhead is structural, not per-service-specific. It is the same on every service. This is what makes shared infrastructure (cached base images, pre-warmed runners, common cache backends) so high-leverage in microservices CI: one infrastructure improvement reduces overhead across all N services simultaneously.

### Reusable workflow templates

The most powerful organisational tool for microservices CI is the reusable workflow template. GitHub Actions reusable workflows let you define a workflow once and call it from many service repos. GitLab CI includes serve the same purpose. CircleCI orbs are the equivalent.

The template defines the structural pipeline: setup steps, common security scans, deploy hooks, notification rules. The service repo defines only what is service-specific: which test command, which artifacts to publish. A typical service-side workflow becomes 10-20 lines of YAML calling the template, instead of the 200-500 lines that would otherwise duplicate across N services.

The cost benefit is twofold. First, when you optimise the template (better caching, faster setup, removed redundant steps), every service benefits immediately. Second, when you add a new check (security scan, license compliance) you add it once in the template rather than 50 times in 50 service repos.

### Template versioning: blast radius

A template that all services use becomes a critical-path system. A bad template change can break CI for every service in the organisation. Mitigation: explicit template versioning. Pin services to specific template versions (in GitHub Actions, uses: org/template-repo/.github/workflows/ci.yml@v1 ). Roll out template upgrades gradually: cut a v2, migrate one service first, watch for issues, then expand to all services over a week or two.

The platform team owns template SLO: any breaking change to a pinned version goes through change-control like any production system. Service teams have predictable behaviour from the template they pin to, with explicit upgrade pressure (security patches need to be adopted within a deadline) but no surprise breakage.

### Shared cache backends

Each service's dependency cache lives separately by default. Each service's Docker layer cache lives separately. Aggregate cache storage grows with services x cache-size-per-service. With 50 services each having a 500MB cache, total CI cache storage is 25GB, which is well past the GitHub Actions 10GB per-repo limit.

The fix is a shared cache backend that lives outside the per-repo cache. S3 with appropriate bucket structure, ECR Pull Through Cache for container images, a shared Artifactory or Verdaccio for language packages. Each service draws from the shared cache, so the "first build pays cold cost" phenomenon happens once per dependency across the whole org rather than once per service.

Operational cost of running a shared cache: typically $50-200/month for the backing storage and any small operating service. Saving on CI compute: typically $300-1,000/month for a 50-service team. Strong payback at any meaningful scale.

### Centralised vs federated CI ownership

Two organisational models for who owns CI in a microservices environment. Centralised: a platform-engineering team owns CI infrastructure, templates, runners, and cost optimisation. Service teams consume the templates and own only their service-specific pipeline content. Federated: each service team owns its own CI end-to-end, including infrastructure choices.

Centralised is structurally cheaper at scale because of the economies of shared infrastructure. The trade-off is service-team autonomy: a service team that wants to use a specific build tool or a specific test framework may be constrained by what the template supports. Federated is more flexible but consistently more expensive: each team reinvents caching, observability, and runner provisioning. Most engineering organisations of 100+ people land on centralised by force of cost discipline.

The hybrid pattern that works: centralised platform owns the structural pipeline and the runner pool; service teams own the test content and the deploy hooks. Service teams contribute back to templates when they need new functionality. Platform team holds the cost dashboard and the SLOs.

### Cross-repo trigger costs

Sometimes services depend on each other in CI: a backend service's changes need to trigger an integration test in the API gateway service. Cross-repo triggers add cost because each linked CI run is real compute. Done well, this is a small line item; done poorly (every commit to service A triggers full integration suites in services B, C, D), it can dominate the bill.

The pattern that scales: contract tests rather than integration tests. Each service publishes its API contract; consumers run a contract test against the published contract, not against a live instance of the producing service. Pact and Spring Cloud Contract are the dominant tools. Contract tests are fast, do not require cross-repo orchestration at CI time, and catch the same incompatibilities that full integration tests catch (with some discipline about contract maintenance).

### Frequently Asked Questions
 Why does microservices CI cost so much? 
Each service has its own CI pipeline. Even minimal pipelines (build, lint, test, deploy) run on every push to that service. With 50 services and 10 pushes per service per day, you have 500 pipeline executions per day with no opportunity for shared work. Compared to a monolith with one pipeline run per push, microservices CI multiplies the per-service cost by N services. The fix is shared CI infrastructure (templates, common steps) and per-service caching, not eliminating the per-service pipelines.
 What is the per-service CI overhead? 
Per-service overhead is the time each pipeline spends on setup tasks unrelated to the service-specific work: container image pull, dependency cache restore, test framework setup, code coverage tooling. Typical setup overhead per pipeline run: 60-180 seconds. Multiplied across N services with M pushes each, this overhead is often 25-40% of total CI minutes for microservices teams. Aggressive base image caching and lightweight runner setup cut it to 10-15%.
 Should microservices share a CI template? 
Yes for the structural pipeline (steps, ordering, deploy targets). Reusable workflows in GitHub Actions, includes in GitLab CI, orbs in CircleCI all support this. The trade-off: a template change propagates to all services, which is good for consistency and risky for blast radius. Pin services to specific template versions, version templates explicitly, and roll out template upgrades gradually rather than as a single org-wide push.
 What is the right CI ownership model for microservices? 
Hybrid: a central platform team owns the CI templates, infrastructure, and shared concerns. Each service team owns the service-specific pipeline content (test definitions, deploy hooks). The platform team is the SRE for CI; the service teams are the consumers. This works because CI infrastructure has economies of scale (one runner pool serves many services) but service-specific concerns are best owned by people close to the service.
 When should microservices teams move to a monorepo? 
When the cost of cross-service coordination exceeds the cost of CI duplication. Monorepos win on shared dependency upgrades, atomic refactors across services, and unified testing of contract changes. Microservices win on independent release cadences, per-team tooling autonomy, and service-team-owned operations. The CI cost difference is real but rarely the deciding factor: it is one input among many, and affected-only patterns close most of the gap on either side.
 
### Related cost components and shapes
 
- Monorepo CI cost 
- Branch protection cost 
- CI/CD caching strategies 
- Cross-region egress cost 
- CI/CD cost for scaleups 
- CI runner cost: hosted vs self-hosted