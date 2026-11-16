# Verified Content: 6 CI/CD Cost Components (cicdcost.com)


---
# PAGE: `build-minutes-cost` (https://cicdcost.com/build-minutes-cost)
Build Minutes Cost in 2026: How Vendors Charge for Compute | CICDCost.com Independent CI/CD cost research, read by teams pricing their pipeline. Sponsor this site → CICDCost.com is an independent comparison resource. Not affiliated with GitHub, GitLab, CircleCI, Buildkite, or any CI/CD vendor. Try our CI/CD calculator 
# Build minutes cost in 2026: every vendor's pricing model 

A build minute looks like a unit. It is not. Each CI vendor counts, rounds, and surcharges differently, and those small rules compound into bills that surprise teams every quarter. This page breaks down what one minute actually costs in 2026 across the seven CI platforms most teams evaluate, with vendor pricing pages cited for each. 

Headline rate (Linux, 2026) 

AWS CodeBuild general1.small is the cheapest at $0.005/min. GitHub Actions and CircleCI sit at $0.006/min for the default Linux class. GitLab and Bitbucket are both at $0.010/min on shared runners. Buildkite charges per user, not per minute. Self-hosted runners on EC2 work out to roughly $0.001-$0.002/min once amortised. 

## What a build minute actually means 

A build minute is the wall-clock duration a job runs on a hosted runner, rounded up by rules that differ per vendor and per plan. The runner clock starts when the job is dispatched to a runner (after queue time), not when you push the commit. Setup steps count: cloning your repo, restoring caches, pulling Docker images. Tear-down counts too. Anything inside the job container before the explicit `run` step is your minutes. 

The first surprise is rounding. GitHub Actions and GitLab round each job up to the nearest whole minute. A 30-second job costs the same as a 59-second job. Pipelines that fan out into many short jobs (which is great for parallelism) pay the rounding tax on every one of them. A 20-job matrix where each takes 35 seconds bills as 20 minutes, not 12. CircleCI bills in credits at one-second granularity on Linux, so the rounding penalty does not apply there. AWS CodeBuild bills per second after a one-minute minimum: same 35-second job bills as 60 seconds. 

The second surprise is the OS multiplier. GitHub Actions applies a 1x factor for Linux, 2x for Windows, and 10x for macOS. So a 10-minute Windows job consumes 20 minutes of your free tier and bills at the Windows rate. A 10-minute macOS job consumes 100 minutes of free tier and bills at $0.48. The multiplier exists because the underlying hardware cost is genuinely different. Linux runs as a container slot on shared compute. Windows requires a full VM with a paid Microsoft licence. macOS, per Apple licensing, must run on physical Mac hardware leased on 24-hour minimums. GitLab applies cost factors rather than flat OS surcharges on its hosted runners: Windows medium consumes at factor 1 (level with standard Linux), while macOS consumes at 6x to 12x, which keeps GitLab competitive for Windows-heavy CI but not structurally cheaper for macOS: see the GitLab compute minutes documentation for the live cost-factor table. 

The third surprise is concurrency caps. Free tiers and lower paid tiers cap the number of concurrent jobs your account can run. GitHub Actions Free allows 20 concurrent jobs across the account. CircleCI Free allows one. Hitting the cap means jobs queue, which extends pipeline wall-clock without lowering the bill. Worse, the queue time pressures teams to upgrade to a higher tier purely to unblock parallelism, which is often the largest single jump in their CI bill. 

## Per-vendor build-minute rates (2026) 

The table below is the headline rate for the lowest-tier paid plan on each platform, with the default Linux runner class. All figures verified against vendor pricing pages, which are linked in the source column. Always re-verify before procurement: pricing is a moving target and we have seen multiple platforms re-price more than once a year. 

| Vendor | Linux/min | Windows/min | macOS/min | Rounding | OS multiplier | Source |
| GitHub Actions | $0.006/min | $0.010/min | $0.062/min | Up to nearest minute, per job | 1x / 2x / 10x | vendor page |
| GitLab CI (SaaS) | $0.010/min | $0.010/min (medium, Beta) | ~$0.06/min effective (6x factor) | Up to nearest minute, per job | Cost factors: 1x Linux/Windows medium, 6-12x macOS | vendor page |
| CircleCI | $0.006/min (Medium) | $0.024/min | $0.12+/min | Per second, paid in credits | Resource-class scaled | vendor page |
| AWS CodeBuild | $0.005/min (general1.small) | $0.018/min | Not offered | Per second, 1-min minimum | Compute-class scaled | vendor page |
| Bitbucket Pipelines | $0.010/min over included | Not offered | Not offered | Up to nearest minute | 1x flat | vendor page |
## Worked example: a 25-developer team in 2026 

Consider a 25-developer team shipping a Node service plus an iOS client. Each developer pushes 5 times per workday on average. Every push triggers a CI run that takes 8 minutes on Linux for lint, unit tests, and integration tests, plus a 12-minute macOS build on the iOS side for the engineers who touch the mobile app (assume 6 of the 25 do). Twenty working days per month. 

Linux usage: 25 devs x 5 pushes/day x 20 days x 8 min = 20,000 min/month. iOS macOS usage: 6 devs x 5 pushes/day x 20 days x 12 min = 7,200 min/month. On GitHub Actions Team plan that is 17,000 billable Linux min after the 3,000 free allowance, plus 7,200 macOS min that consumes 72,000 free-tier units (well past the cap). Net bill: 17,000 x $0.006 + 7,200 x $0.062 = $102 + $446.40 = roughly $548/month for compute, plus the per-seat licence at $4 x 25 = $100. Total about $648/month. 

Same team on GitLab Premium: cost factors hit the macOS side here too. 20,000 Linux min at factor 1 plus 7,200 macOS min at factor 6 = 63,200 compute minutes, less the included 10,000 on Premium, leaves 53,200 billable at $0.010 = $532 in compute. Plus per-seat at $29/user x 25 = $725. Total about $1,257/month. GitLab's compute is competitive for Linux and Windows work, but macOS cost factors and the steep per-seat make it the most expensive option for this mixed Linux + iOS team. 

Same team on self-hosted GitHub Actions runners on a pair of c5.2xlarge spot instances ($0.10/hr each): 1,440 hours/month x $0.20 = $288/month for compute, with no per-minute charge from GitHub (the platform fee announced for March 2026 was postponed and never took effect). Total compute $288, plus the per-seat at $4 x 25 = $100. Total about $388/month, but you now own the runners and the on-call burden. The savings are real but conditional on having an engineer who can keep the runner controller healthy. 

## The hidden minute multipliers 

Headline rates only describe what one minute on the runner costs. They do not describe what your bill becomes once you account for retries, queue time, and tier upgrades that exist solely to lift concurrency caps. 

Retries are the largest hidden multiplier. A flake rate of 10% (typical for medium-maturity teams per published industry research from Snyk and others) means roughly one in ten builds is re-run, doubling the minute spend on that branch. Quarterly bills typically come in 8-15% over the un-retried estimate purely because of flake. The fix is structural (deflake, quarantine, retry-only-the-failed-test) not financial, but the cost line is real. We dig into this on the failed-build rerun cost page . 

Queue time is not billed but it costs developer time. The economic value of cutting queue time often exceeds the savings of optimising minute spend. We quantify that on the developer wait time cost page using DORA and Stack Overflow Developer Survey salary anchors. 

Tier-upgrade cost is the most sneaky. A team on GitHub Actions Team can hit the 60 concurrent job cap once it scales past about 30 active developers. The path to lift the cap is GitHub Enterprise, which jumps the per-seat from $4/user to $21/user. The CI bill effectively quintuples for reasons unrelated to minute consumption. Plan ahead by knowing your concurrent-job ceiling now and the next-tier price. 

## Reading vendor pricing tables correctly 

Three rules. First, ignore the "included minutes" in marketing copy until you compare them across vendors at the same plan tier. Free-tier numbers are not equivalents (GitHub Free includes 2,000, GitLab Free includes 400, and GitLab applies cost factors rather than OS multipliers). Second, find the per-OS rate. Many marketing pages bury Windows and macOS pricing two clicks deep. Third, find the concurrent-job cap and the price of lifting it. The minute rate is rarely the binding constraint at scale; the concurrency cap is. 

Good vendor pricing pages to bookmark: the live GitHub plans page with the "Compare plans" expand, the GitLab pricing with the CI minutes line item, the CircleCI pricing with its credits calculator, and the AWS CodeBuild pricing page with the compute-type matrix. We re-verify our table against these monthly. 

## Frequently Asked Questions 
What is a build minute in CI/CD? 
A build minute is the unit most CI/CD vendors use to bill compute. It is the wall-clock time a job runs on a hosted runner, rounded up by vendor-specific rules. GitHub Actions and GitLab round up to the nearest minute per job. AWS CodeBuild bills per second after a 1-minute minimum. CircleCI bills in credits at a granularity of one second on Linux. The unit is conceptually identical across vendors but the rounding cost adds up at scale. 
Why do macOS minutes cost so much more than Linux? 
Hosted macOS runners require Apple silicon hardware that vendors must own and physically rack. Apple's licensing terms restrict cloud virtualisation of macOS to a maximum of two VMs per Mac and 24-hour minimum lease windows, which makes oversubscription impossible. The hardware cost per macOS minute is genuinely 8-10x higher than a Linux container slot. GitHub Actions charges $0.062/min for macOS in 2026 (down from $0.080 after the January cut), versus $0.006/min for Linux: roughly a 10x multiplier. 
How are build minutes counted across vendors? 
GitHub Actions counts wall-clock time per job, rounds up to the nearest minute, and applies an OS multiplier (1x Linux, 2x Windows, 10x macOS) before deducting from your free tier. GitLab counts compute minutes per job, also rounds up, and applies a cost factor by runner type (1x standard Linux and Windows medium, 6x to 12x macOS). CircleCI converts minutes into credits at a per-resource-class rate (a Medium Linux machine consumes 10 credits/min). AWS CodeBuild bills per second after a 1-minute minimum. Buildkite charges per user for the platform; its hosted agents bill per vCPU-minute ($0.004 Linux) and self-hosted Buildkite agents have no minute meter. 
What is an OS multiplier? 
An OS multiplier is a factor that vendors apply to billed minutes based on the runner operating system. GitHub Actions applies 1x for Linux, 2x for Windows, and 10x for macOS. So a 10-minute Windows job consumes 20 minutes of your free tier and is billed at $0.10 (10 minutes at $0.010/min). GitLab applies cost factors by runner type on its hosted runners (1x standard Linux and Windows medium, 6x to 12x macOS). The multiplier reflects underlying hardware cost, not actual job runtime. 
How can I reduce my build-minute spend? 
Five techniques typically deliver the biggest savings. Cache dependencies (npm, pip, Maven, Cargo) to skip 30-60% of cold install time. Use path filters so only changed services trigger CI. Run quick checks before slow ones and fail fast. Move heavy CI workloads to self-hosted runners on EC2 spot instances if your monthly minute spend exceeds $100. Drop the macOS portion of your matrix to a nightly cron rather than per-PR if you do not actually ship iOS daily. 

## Related cost components 

- Runner cost: hosted vs self-hosted by the hour 
- Artifact storage cost across vendors 
- Parallelism cost: when concurrency blows the budget 
- Log retention cost vendors hide 
- Cross-region egress cost 
- Build cache savings: real numbers 
Want vendor-by-vendor totals at your exact build minute count? Use cicdcalculator.com for the side-by-side estimator. 

Updated 2026-08-04 


---
# PAGE: `runner-cost` (https://cicdcost.com/runner-cost)
CI Runner Cost in 2026: Hosted vs Self-Hosted by the Hour | CICDCost.com Independent CI/CD cost research, read by teams pricing their pipeline. Sponsor this site → CICDCost.com is an independent comparison resource. Not affiliated with GitHub, GitLab, CircleCI, Buildkite, or any CI/CD vendor. Try our CI/CD calculator 
# CI runner cost in 2026: hosted, self-hosted, and the hourly math 

Vendors quote per-minute rates because per-minute makes the headline number small. Per-hour math makes the comparison to self-hosted runners visible. This page converts every major vendor's per-minute rate into a per-hour equivalent and benchmarks each against the EC2 instance class you would actually pick if you ran the runner yourself. 

Headline at a glance (2026) 

Hosted Linux runners run $0.30 to $0.36 per hour of active build time. Self-hosted t3.medium on EC2 on-demand costs $30/month for 720 hours (about $0.04/hour amortised). Spot c5.4xlarge lands near $130/month for the same hours. The crossover where self-hosted saves money sits around 5,000 monthly minutes on GitHub Actions for a $30 runner (no platform fee applies to self-hosted runners; the one announced for March 2026 was postponed). 

## Hosted runner hourly equivalents 

Convert the per-minute rate to per-hour and the picture changes. GitHub Actions Linux at $0.006/min is $0.36/hour. CircleCI Medium Linux is the same. AWS CodeBuild general1.small is $0.30/hour. GitLab CI shared runners at $0.010/min equate to $0.60/hour. Buildkite's hosted agents bill $0.004 per vCPU-minute on Linux, about $0.48/hour for a 2-vCPU agent and $0.96/hour for 4 vCPU. macOS jumps the figure dramatically: GitHub Actions macOS at $0.062/min is $3.72/hour, CircleCI macOS is $7.20/hour or higher. 

Per-hour numbers are only meaningful when a runner is busy. Hosted runners bill only while a job is executing. A runner that sits idle for 23 hours of the day costs nothing on a hosted plan, which is why hosted runners win at low volume. The whole comparison flips at higher utilisation, where always-on self-hosted runners amortise across many jobs and the per-hour cost falls toward EC2's sticker price. 

## Self-hosted on AWS EC2 

The most common self-hosted runner is a single EC2 VM with the GitHub Actions runner binary installed. Sized appropriately, this is cheaper than the hosted equivalent above a fairly low minute threshold. EC2 pricing for the classes that suit CI workloads, sourced from the AWS EC2 on-demand pricing page : 

| Instance | vCPU / RAM | On-demand /hr | On-demand /mo | Spot typical /hr | Best for |
| t3.medium | 2 vCPU / 4 GB | $0.0416 | $30 | $0.012 | Linters, small unit suites |
| m5.large | 2 vCPU / 8 GB | $0.096 | $69 | $0.029 | Mid-size builds, mixed loads |
| m5.xlarge | 4 vCPU / 16 GB | $0.192 | $138 | $0.058 | Compile-heavy, mid-tier integration |
| c5.2xlarge | 8 vCPU / 16 GB | $0.34 | $245 | $0.10 | CPU-bound, parallel test suites |
| c5.4xlarge | 16 vCPU / 32 GB | $0.68 | $490 | $0.17 | Heavy monorepo, video, ML |
| m6g.xlarge (Arm) | 4 vCPU / 16 GB | $0.154 | $111 | $0.046 | Arm-native builds, cost-sensitive |
Storage adds modest cost: an 80 GB gp3 root volume is about $6.40/month. A CloudWatch agent for runner observability costs roughly $1-3/month per instance. Outbound data transfer is the line that bites unpredictably (we cover this in detail on the cross-region egress cost page ). Reasonable rule: budget 10-15% on top of the EC2 sticker price for surrounding services. 

## Spot vs on-demand: the 70% discount 

Spot pricing typically runs 60-75% off on-demand for CI-suited instance families. The catch is interruption: AWS reclaims the spot capacity with a 2-minute warning. CI workloads are interruption-tolerant when the orchestrator handles retries automatically, which is why spot is a particularly good fit for CI compared to most other workloads. We dedicate the spot instance CI cost page to this in depth. 

Worked example: a 50-developer team running roughly 40,000 build-minutes/month wants to migrate from GitHub Actions hosted to self-hosted. On hosted, 40,000 min x $0.006 = $240/month for compute (no platform fee applies). On a single c5.4xlarge spot at $0.17/hour with 60% utilisation: 720 hours x 0.6 x $0.17 = $73/month plus surrounding services, call it $90 total. Saving roughly $150/month. Multiply by 12 for an annual saving of $1,800, less the engineer-time cost of running an Actions Runner Controller cluster (call it 2 days/year at fully-loaded $1,500/day): the labour cost slightly exceeds the compute saving at this volume. 

That math flips dramatically once monthly minutes pass 100,000 or so, where the compute saving from self-hosting overwhelms the labour overhead. Below 25,000 minutes it does not pay back. The middle band is the area where the answer depends on your team's appetite for owning runner infrastructure. 

## Always-on vs ephemeral runners 

Always-on runners are constantly available and constantly costing. Their cost is the EC2 sticker price for 720 hours a month, regardless of how busy the runner is. They are the simplest pattern: install the runner binary, set it as a service, walk away. Ephemeral runners spin up per job and tear down after, which means you only pay EC2 while the job runs. The cost story is similar to hosted runners (pay-when-busy) but at self-hosted instance prices. 

Always-on wins on simplicity and on cold-start latency: a job dispatched to an idle runner starts in seconds. Ephemeral wins on cost when utilisation is low: if your runners are busy 5% of the day, paying for the 95% idle time is wasteful. Most teams in the 25-100 dev range land on a hybrid: a small fixed pool of always-on runners for the typical load plus an ephemeral autoscaler (ARC, Karpenter) that spins up extra capacity when the queue grows. We cover the autoscaling pattern on the ephemeral runner cost page . 

## macOS runners: the special case 

macOS pricing is structurally different because Apple licensing forbids cloud virtualisation beyond two VMs per Mac on a 24-hour minimum lease. Hosted macOS runners cost what they cost because vendors must own the Mac hardware and rack it physically. Self-hosted macOS means buying Mac minis or M2 Pros and running them in your office or in a dedicated colo. 

A Mac mini M2 with 16 GB is about $700 from Apple . Three-year amortisation puts the hardware cost at $19/month. Power draw idle is about 6-9 watts; under load up to 25 watts. At $0.15/kWh and 40% utilisation, monthly electricity is roughly $1.50-3.00. So a self-hosted Mac mini costs about $25/month total. The same volume on hosted GitHub Actions macOS at $3.72/hour ($0.062/min) and about 70 hours/month of build time is roughly $260/month. Mac mini self-hosting pays back in about 3 months even for a small iOS team, assuming you have somewhere safe to plug it in. 

The trade-off is that physical Mac minis must be physically maintained, the macOS updates must be applied, the runner binary must be kept current, and the office network must reach the GitHub or GitLab orchestrator. For high-volume iOS teams the maths is still compelling. For low-volume teams (under 50 builds/month) the simpler hosted option is fine. 

## Frequently Asked Questions 
What does a hosted CI runner cost per hour? 
GitHub Actions hosted Linux at $0.006/min works out to $0.36/hour. macOS at $0.062/min is $3.72/hour. CircleCI Medium Linux at $0.006/min is also $0.36/hour. AWS CodeBuild general1.small at $0.005/min is $0.30/hour. These are billable when a job is running, not when the runner is idle, which is why the per-hour math only matters when you compare to always-on self-hosted. 
What does a self-hosted runner cost on EC2? 
On AWS EC2 in 2026: t3.medium on-demand is $0.0416/hour ($30/month always-on). m5.large is $0.096/hour ($69/month). c5.4xlarge is $0.68/hour ($490/month). Spot pricing typically runs 60-75% off on-demand, so a c5.4xlarge spot can land at $0.17/hour. Add roughly $5-10/month per runner for storage and CloudWatch. 
When does self-hosting save money on CI runners? 
The break-even on a t3.medium self-hosted runner ($30/month always-on) versus GitHub Actions Linux ($0.006/min) is about 5,000 minutes/month. No platform fee applies: the $0.002/min charge GitHub announced for March 2026 was postponed and never took effect. A c5.4xlarge spot at $130/month breaks even at about 22,000 minutes. Buildkite and self-managed GitLab CE likewise charge nothing per minute for self-hosted compute. 
What is an ephemeral CI runner? 
An ephemeral runner is a single-job runner that is created when a build queues and destroyed when the build finishes. Each job runs on a fresh, clean container or VM with no state from the previous job. Ephemeral patterns are common with Actions Runner Controller (ARC) on Kubernetes, GitLab Kubernetes Executor, or scripts triggered by SQS. They cost more in startup latency than always-on runners but eliminate idle compute charges, which is the cost case for using them above ~50,000 minutes/month. 
Are macOS self-hosted runners cost-effective? 
A Mac mini M2 (16GB) costs about $700 once. Electricity at $0.15/kWh for an always-on Mac mini is about $4-6/month. Cloud macOS on AWS EC2 mac1.metal is $1.083/hour ($792/month). Hosted macOS on GitHub Actions at $0.062/min equals $3.72/hour. A team doing 200 macOS builds per month at 12 minutes each pays $149 on GitHub Actions versus $5-7 in electricity on a Mac mini. The Mac mini pays for itself in about 5 months at that volume. 

## Related cost components 

- Build minutes cost across vendors 
- Ephemeral runner cost: spin-up-spin-down economics 
- Spot instance CI cost: 70% off self-hosted 
- Parallelism cost: when concurrency blows the budget 
- Self-hosted vs cloud break-even analysis 
- Jenkins self-hosted cost guide 
For vendor-by-vendor totals at your runner mix, use cicdcalculator.com . 

Updated 2026-07-06 


---
# PAGE: `artifact-storage-cost` (https://cicdcost.com/artifact-storage-cost)
CI/CD Artifact Storage Cost in 2026: GB Pricing Across Vendors | CICDCost.com Independent CI/CD cost research, read by teams pricing their pipeline. Sponsor this site → CICDCost.com is an independent comparison resource. Not affiliated with GitHub, GitLab, CircleCI, Buildkite, or any CI/CD vendor. Try our CI/CD calculator 
# CI/CD artifact storage cost in 2026: per-GB pricing and retention math 

Storage is the line item teams ignore until the bill arrives. CI artifacts (build outputs, test reports, log bundles, coverage data) accumulate quietly. The default retention is generous, the default upload is "everything", and the per-GB price is small enough to feel rounding-error. At 200 builds/day across 30 services for two years, that rounding error becomes a four-figure monthly line. This page walks through what each vendor charges, what default retention does to your bill, and the two levers that consistently cut artifact storage spend 70-90%. 

Headline at a glance (2026) 

Vendor-hosted artifact storage costs roughly $0.25/GB-month across GitHub Actions, GitLab, and CircleCI. The same data on AWS S3 Standard is $0.023/GB-month, an 11x markup for the convenience of vendor-hosted artifacts. Default retention is 30-90 days everywhere; cutting to 7-14 days for non-release artifacts typically saves 70-85% of monthly storage spend. 

## Per-vendor artifact storage rates (2026) 

| Vendor | Included | Overage /GB-mo | Default retention | Egress fee | Source |
| GitHub Actions | 500 MB Free / 2 GB Team / 50 GB Enterprise | $0.25 | 90 days | No artifact download fee listed | vendor docs |
| GitLab CI | 5 GB shared with repository storage | $0.50 (Storage Add-On, $5 per 10 GiB) | 30 days | Included | vendor docs |
| CircleCI | 15 GB workflow storage on Free | ~$0.25 via credits (420 credits/GB) | 15 / 30 days | Bundled | vendor pricing |
| Buildkite | 10 GB hosted artifacts | $0.10 | 30 days | Included | vendor pricing |
| AWS S3 Standard (DIY) | 5 GB Free Tier (12 mo) | $0.023 | You set | $0.09/GB out | vendor pricing |
| AWS S3 Intelligent-Tiering | n/a | $0.012-$0.023 | You set | $0.09/GB out | vendor pricing |
## Worked example: a 30-service backend platform 

A platform team running 30 microservices, each with a CI pipeline that uploads roughly 80 MB of artifacts per build (test reports, coverage, the built binary, log bundles). Average 15 builds/service/day across 20 working days = 9,000 builds/month per service x 30 services... no, scale that down: 15 builds/service/day x 20 days x 30 services = 9,000 builds/month total. Storage created per month: 9,000 x 80 MB = 720 GB. 

With the GitHub Actions default 90-day retention, peak storage is 720 GB x 3 months = 2,160 GB stored at any given time once steady-state is reached. Less the 2 GB Team-tier inclusion: 2,158 GB billable at $0.25/GB-month = $540/month. The same workload on S3 Standard would be $50/month. Same workload with retention dropped to 14 days: peak storage 720 GB x (14/30) = 336 GB stored, billed at $0.25 x 335 = $84/month on GitHub Actions, or $8/month on S3. 

Two changes (cut retention from 90 days to 14 days, push to S3 instead of vendor-hosted) take the bill from $540/month to $8/month, a 67x reduction. Most teams in the 25-100 dev range have a meaningful artifact storage line and have not made either change. 

## Retention policy: the simplest lever 

Retention can be set globally on most platforms but the per-workflow override is more powerful. GitHub Actions lets you set retention-days per artifact upload step. GitLab supports artifacts.expire_in per job. CircleCI does not expose retention as a per-job setting (it is plan-wide), which is one of the reasons larger teams drift toward GitHub Actions or self-hosted GitLab. 

The triage to use: classify artifacts into "deployable" (release binaries, signed bundles, container images you might roll back to), "diagnostic" (test reports, coverage, log bundles you keep for trend analysis), and "intermediate" (build caches, object files, anything you never look at after the next push). Set retention to 365 days for deployable, 30-60 days for diagnostic, 1-7 days for intermediate. Most teams have far more intermediate than they realise. 

## When to push to S3 instead 

The vendor-hosted artifact convenience comes with an 11x markup on per-GB-month versus S3. The break-even where rolling your own becomes worth the integration effort is roughly $50/month in vendor artifact spend, which corresponds to about 200 GB of stored artifacts at the GitHub Actions rate. Above that threshold, an S3 bucket with a lifecycle policy (delete after N days, transition to Intelligent-Tiering after 30) plus a small upload step in CI is straightforward to maintain. 

The integration cost is real: you need an OIDC trust between your CI and AWS, an IAM policy for the bucket, a CI step that uploads with the right key naming convention, and a cleanup script if your lifecycle rules are not enough. Most teams take 1-2 days to set up and then forget about it. The S3 lifecycle policy documentation is on the AWS docs page. 

## The bandwidth trap 

Storage is the headline. Bandwidth is the gotcha. GitHub's Actions billing documentation lists no artifact download fee, and package downloads made by a workflow with GITHUB_TOKEN do not count against GitHub Packages transfer. S3 charges $0.09/GB out to the public internet. If you fan out artifact downloads to many concurrent jobs (a common pattern for matrix testing where each job needs the build output), the egress can outpace storage. Cache locally in the runner where possible, and consider using a CloudFront or vendor-CDN distribution if your CI pulls the same large artifacts repeatedly. 

Cross-region downloads add a separate AWS egress charge ($0.02/GB inter-region transfer plus the destination egress). Pin your CI artifacts to one region and pull from runners in the same region wherever the workflow allows. We cover the egress dimension in detail on the cross-region egress cost page . 

## Container registry storage: a sibling line 

If your CI builds container images, those images live in a container registry (GHCR, GitLab Container Registry, ECR, Docker Hub). The pricing math is similar: per-GB-month, with vendor markup over raw S3. ECR Standard is $0.10/GB-month. GitHub Container Registry is $0.25/GB-month for private images (free for public). Docker Hub paid plans bundle storage with seat counts. 

The cost driver is layer churn. Container images pile up image manifests as each build produces a new tag. A retention policy on the registry that keeps only the most recent N tags per service plus all tags matching v* for releases is often sufficient. ECR has lifecycle policies built in. GHCR exposes retention through GitHub Packages settings. Either way, the discipline is the same as artifact storage: classify, retain, prune. 

## Frequently Asked Questions 
How much does GitHub Actions artifact storage cost? 
GitHub Actions includes 500 MB of artifact storage on the Free plan, 1 GB on Pro, 2 GB on Team, 50 GB on Enterprise Cloud (shared with GitHub Packages). Beyond the included quota, storage costs $0.008/GB per day, about $0.25 per GB-month. GitHub's Actions billing documentation lists no separate charge for downloading artifacts; data transfer is metered for GitHub Packages, and package downloads made by a workflow with GITHUB_TOKEN do not count against it. The default retention is 90 days, which means a 200 MB build artifact created on every push to main at 30 builds/day adds about 6 GB of storage a day, roughly 180 GB within a month. 
How long are CI artifacts retained by default? 
GitHub Actions retains artifacts for 90 days by default and you can lower this per-workflow or per-org. GitLab retains job artifacts for 30 days by default with similar overrides. CircleCI retains workflow storage for 30 days on Performance plan, 15 days on Free. Buildkite hosted artifacts default to 30 days. Lowering the default to 7 or 14 days is the single largest storage cost lever you have. 
What is the cheapest CI artifact storage approach? 
Pushing artifacts to your own S3 bucket is typically cheaper at scale than using vendor-hosted artifact storage. S3 Standard is $0.023/GB-month, almost 11x cheaper than GitHub Actions storage at $0.25/GB-month. S3 Intelligent-Tiering with auto-archive after 30 days drops you to $0.012/GB-month for older artifacts. The trade-off is that you maintain the IAM policy, lifecycle rules, and integration code yourself. 
What artifacts do I actually need to keep? 
Three categories: deployable build outputs (release binaries, container images, signed bundles) need long retention, often 1+ years. Test reports and coverage data need 30-90 days for trend analysis. Intermediate build artifacts (object files, dependency caches, log dumps) can live 1-7 days. Most teams accidentally apply the longest retention to everything because the platform default is 90 days. Triaging by category typically cuts artifact storage 70-90%. 
Does Docker image storage count as CI artifact storage? 
It depends on where you push them. If your CI builds Docker images and pushes them to GitHub Container Registry, GitLab Container Registry, or Docker Hub, that storage is metered separately from CI artifact storage. GHCR for example offers 500 MB free for private packages then $0.25/GB-month. Public images are free at GHCR and Docker Hub. Rule of thumb: container registry storage is roughly the same per-GB-month price as artifact storage, and the same retention discipline applies. 

## Related cost components 

- Log retention cost vendors hide 
- Cross-region egress cost 
- Build cache savings: real numbers 
- Build minutes cost across vendors 
- CI runner cost: hosted vs self-hosted 
- 12 ways to cut your CI bill 
For a vendor-by-vendor monthly total at your storage volume, plug numbers into cicdcalculator.com . 

Updated 2026-11-14 


---
# PAGE: `parallelism-cost` (https://cicdcost.com/parallelism-cost)
CI/CD Parallelism Cost in 2026: Concurrent Jobs and Tier Math | CICDCost.com Independent CI/CD cost research, read by teams pricing their pipeline. Sponsor this site → CICDCost.com is an independent comparison resource. Not affiliated with GitHub, GitLab, CircleCI, Buildkite, or any CI/CD vendor. Try our CI/CD calculator 
# CI/CD parallelism cost in 2026: when concurrent jobs blow your budget 

Parallelism is the single most misunderstood cost lever in CI. Engineers reach for it to make builds faster. Finance teams discover that "more parallel jobs" doubled the bill without obviously delivering 2x feedback loop speed. Both views are partly right. This page breaks down what parallelism actually costs at each major vendor in 2026, where the concurrent-job caps bite, and the test-splitting maths that decide whether throwing more jobs at the problem saves time, money, both, or neither. 

The core insight 

Parallelism does not save minutes. A 60-minute job split into 10 parallel jobs of 6 minutes each consumes the same 60 minutes of billable compute. What it saves is wall-clock time, which is paid in developer-minutes-of-waiting, not vendor-dollars. The economic justification for parallelism is almost always the developer time, not the compute bill. 

## Per-vendor concurrent-job caps 

Each vendor caps concurrent jobs per plan. The cap is enforced at the account level (or organisation level on enterprise plans), not per workflow, which means a busy hour can starve normally-quick pipelines as the cap fills with long-running jobs. Hitting the cap does not fail anything; it queues. 

| Vendor | Plan | Concurrent jobs | Lift cap by | Source |
| GitHub Actions | Free | 20 across account | Upgrade to Team ($4/user) | vendor docs |
| GitHub Actions | Team | 60 across account | Upgrade to Enterprise | same |
| GitHub Actions | Enterprise | 500 across org (50 macOS) | Self-hosted runners (no cap) | same |
| GitLab CI (SaaS) | Free | No per-plan cap published (400 compute min/month) | Self-hosted runners (no cap) | vendor docs |
| CircleCI | Free | 30 | Performance (from $15/mo) | vendor pricing |
| CircleCI | Performance | 80 | Scale (custom) | same |
| Buildkite | Free (up to 5 users) | 10 concurrent jobs | Pro ($30/user/mo), 10 agents included then $3.50/agent | vendor pricing |
| AWS CodeBuild | n/a | Configurable per-account quota | Service quota request to AWS | vendor docs |
## Wall-clock vs minute-spend: the trade you are actually making 

Take a 60-minute integration test suite. Run it as one job and your wall-clock is 60 minutes, billable minutes are 60. Run it as 10 parallel jobs of 6 minutes each (with perfect splitting) and your wall-clock is 6 minutes, billable minutes remain 60. The compute cost is identical; the developer cost is 54 minutes saved per pipeline run. 

What actually happens in practice is worse than perfect splitting. Static splitting (divide files into 10 buckets) leaves long-tail buckets where one bucket runs 10 minutes and nine finish in 4 minutes. Wall-clock = 10 minutes, billable = 9 buckets x 4 + 1 x 10 = 46 minutes. You saved compute too because the 9 short buckets finished early and stopped billing. Timing-based splitting closes the long-tail by sizing each bucket to roughly equal historical duration. CircleCI's circleci tests split --split-by=timings is the canonical example. 

Then there is the rounding tax. GitHub Actions and GitLab round each job up to the nearest minute. Twenty parallel jobs of 35 seconds each bills as 20 minutes, not the 11.7 minutes the work actually consumed. Parallelism plus rounding inflates the bill. CircleCI bills per second on Linux which sidesteps the issue. We covered this in detail on the build minutes cost page . 

## When tier upgrades exist solely to lift the parallelism cap 

The most expensive consequence of parallelism is the tier upgrade it forces. CircleCI's Free plan is the canonical example: it allows 30 concurrent jobs but offers no credit overage, so any team that outgrows the 30,000 monthly credits must upgrade to Performance (from $15/month, 5 active users included) regardless of concurrency needs. The upgrade is functional, not minute-driven. GitHub Actions Team to Enterprise is the same story at larger scale: a 30-developer team hitting the 60 concurrent job cap on Team must move to Enterprise at $21/user, paying $510 more per month for the same build volume. 

If you are evaluating CI vendors at any meaningful scale, build a sizing chart that shows: (a) the concurrent job cap at each plan, (b) the cost to lift the cap to the next tier, (c) your projected concurrent-job needs in 12 and 24 months. Vendors price the "steep concurrency cliff" into their plan tiers because they know teams will pay to avoid queue time once developers complain. 

## Self-hosted runners as the parallelism escape hatch 

Self-hosted runners do not count against vendor concurrent-job caps on most platforms. GitHub Actions self-hosted runners only count against your own infra capacity. GitLab CE/EE self-managed has whatever capacity you provision. This is the underrated reason teams move to self-hosted: not to save compute dollars but to escape the parallelism tier-upgrade tax. 

A 50-developer team on GitHub Actions Team paying $200/user/month in seat costs has one bad reason to upgrade to Enterprise (the 60-job cap) and one expensive way around it (Enterprise at $21/user x 50 = $1,050/month, an extra $850). A self-hosted runner pool on a Kubernetes cluster with Actions Runner Controller provides essentially unlimited parallelism for the price of the underlying compute. We dig into the autoscaling pattern on the ephemeral runner cost page . 

## Decision framework: more parallelism or just one bigger machine? 

Parallelism trades developer-time for setup complexity and the rounding tax. A larger runner trades developer-time for raw compute cost. The right choice depends on the workload shape. 

Workloads that parallelise well (test suites with thousands of independent unit tests, integration tests against ephemeral infrastructure, browser-based E2E tests across browser/OS combinations) reward parallelism heavily. Workloads that do not (a single large compile step, a serial DB migration, an integration test that mutates shared state) reward a beefier single runner instead. 

The cost-effective rule for typical web-service CI: parallelise unit and integration tests across 4-8 jobs, keep the build step serial on a c5.2xlarge, hold E2E tests on a separate matrix that runs only on main-branch merges. This profile minimises both the rounding tax and the wall-clock for the average PR. We give a worked example for the relevant team sizes on the scaleup CI cost page . 

## Frequently Asked Questions 
What are concurrent-job caps in CI? 
A concurrent-job cap is the maximum number of CI jobs your account can run simultaneously across all workflows. GitHub Actions Free is capped at 20 concurrent jobs across the account. Pro is 40, Team is 60 and Enterprise is 500, with macOS capped at 5 (50 on Enterprise). CircleCI Free allows 30 concurrent jobs, Performance plan 80. Buildkite Free allows 10 concurrent jobs. GitLab does not publish a per-plan concurrent-job cap for its hosted runners; its plans are metered in compute minutes instead (400 a month on Free). Hitting the cap means jobs queue rather than fail; queue time grows and developer feedback slows. 
Does adding more parallelism save CI minutes? 
No. Total minutes consumed are roughly the same whether you run 1 job for 60 minutes or 60 jobs for 1 minute each. Parallelism cuts wall-clock time, not billable minutes. The cost case for parallelism is the developer-time saving from faster feedback, not the compute saving. The exception is when parallelism enables better failure isolation (a flake in test bucket 3 of 60 only re-runs that bucket on retry, not all 60), which can produce real minute savings on flaky test suites. 
What does CircleCI's Performance plan add for parallelism? 
CircleCI Performance plan (from $15/month, including 5 active users and 30,000 credits) lifts the concurrent-job cap from 30 (Free) to 80 and unlocks credit overage at $0.0006/credit ($15 per 25,000). The Performance plan is the price most CircleCI customers actually pay because the Free plan has no way to buy additional credits once the 30,000 monthly allowance runs out. Compare credits-per-month vs your needs on the CircleCI pricing page. 
How do I split a test suite for parallel CI? 
Three patterns. Static splitting: divide tests by file count or naming convention into N buckets, run each bucket on a separate job. Simple to set up, can be uneven if some files take much longer than others. Timing-based splitting: use historical test timing data to split into roughly equal-duration buckets. CircleCI's circleci tests split, GitHub Actions matrix with a custom split script, or Knapsack Pro all do this. Dynamic splitting: a coordinator hands out tests to runners as they free up. Highest utilisation, most complex to set up. 
When does adding more parallelism stop helping? 
When the slowest single test sets the floor. If your suite has a 30-minute integration test you cannot subdivide, no amount of parallelism brings the wall-clock below 30 minutes. Amdahl's law applies: maximum speedup is limited by the serial portion. The practical cap for most CI suites is 8-16 parallel jobs; beyond that, queue management overhead and the long-tail test problem dominate. 

## Related cost components 

- Build minutes cost across vendors 
- CI runner cost: hosted vs self-hosted 
- Matrix build cost strategy 
- Developer wait time cost 
- Ephemeral runner cost 
- CI/CD cost for scaleups 
For vendor totals at your concurrency profile, use cicdcalculator.com . 

Updated 2026-11-14 


---
# PAGE: `log-retention-cost` (https://cicdcost.com/log-retention-cost)
CI/CD Log Retention Cost in 2026: Storage Bills Vendors Hide | CICDCost.com Independent CI/CD cost research, read by teams pricing their pipeline. Sponsor this site → CICDCost.com is an independent comparison resource. Not affiliated with GitHub, GitLab, CircleCI, Buildkite, or any CI/CD vendor. Try our CI/CD calculator 
# CI/CD log retention cost in 2026: where the bytes actually go 

Build logs feel free because the CI vendor includes them. They are not free; the vendor priced 30-90 days of retention into your seat cost and your minute cost. The bill becomes visible the moment you decide you need longer retention than the platform default, or you ship logs to your observability stack for cross-system correlation. This page maps the per-vendor default, the per-language log volume, and the per-GB cost of every common downstream destination. 

Headline at a glance (2026) 

Vendor-included log retention covers the first 30-90 days at no extra charge. Shipping CI logs to Datadog adds $0.10/GB ingestion plus $50-150/month retention for typical volume. Loki self-hosted on Kubernetes with S3 backing handles the same workload for $80-150/month total. The decision is rarely about money; it is about whether you need cross-system correlation badly enough to take on log shipping. 

## Per-vendor log retention defaults 

Each vendor includes log retention as part of the plan. None expose a per-workflow override the way they do for artifact retention. If you need different retention you are shipping logs out, not configuring the vendor. 

| Vendor | Plan | Log retention | Configurable? | Source |
| GitHub Actions | Free / Team / Enterprise | 90 days | No (org-wide only) | vendor docs |
| GitLab CI (SaaS) | Free | 30 days | Yes (per-job artifacts.expire_in for downloadable logs) | vendor docs |
| CircleCI | Performance | 30 days | No | vendor pricing |
| Buildkite | Free / Pro / Enterprise | 30 days / 90 days / 1 year build retention | Yes (export to your own storage) | vendor pricing |
| AWS CodeBuild | n/a | Default 14 days in CloudWatch Logs | Yes (CloudWatch retention setting) | vendor docs |
## Per-language build log volume 

Languages and build tools differ wildly in how chatty they are. Java/Gradle is verbose by default. Go is terse. Node sits in the middle, with the test framework adding the most variability. The ranges below are for a typical build: install dependencies, compile or transpile, run unit tests, output coverage and a results summary. Verbose flags inflate considerably; failed builds with stack traces typically double the volume. 

| Language / build tool | Per-build typical log size | Verbose flags | Per failed build |
| Node / npm / Vitest | 200-800 KB | 1-4 MB | 500 KB-2 MB |
| Python / pytest | 300 KB-1.2 MB | 2-8 MB | 800 KB-2.5 MB |
| Java / Maven | 500 KB-2 MB | 5-15 MB | 1.5-5 MB |
| Java / Gradle | 800 KB-3 MB | 10-25 MB | 2-8 MB |
| Go / go test | 100-400 KB | 500 KB-2 MB | 200 KB-1 MB |
| .NET / dotnet test | 400 KB-1.5 MB | 2-6 MB | 1-3 MB |
| Rust / cargo test | 600 KB-2 MB | 3-8 MB | 1.5-4 MB |
| iOS / xcodebuild | 2-8 MB | 15-40 MB | 5-15 MB |
| Android / Gradle + emulator | 3-10 MB | 20-50 MB | 8-20 MB |
Worked example: a Java backend team running 800 builds/day across services with Gradle as the build tool. Average 1.5 MB per build = 1.2 GB/day = 36 GB/month of build logs. Verbose CI on PRs adds another 25%. Failed-build amplification at a 10% flake rate adds another 15%. Realistic monthly log volume: 50-55 GB. 

## Shipping logs to Datadog 

Datadog is the most common destination for CI logs because most teams already pay for Datadog for application observability. The integration is well-documented; the cost is not always obvious. Datadog charges by ingestion volume and by retention duration, separately. The Datadog Log Management pricing page is the authoritative source. Headline numbers: $0.10/GB ingested, plus retention at $1.27/million events for 7 days, $1.59/million for 15 days, $2.50/million for 30 days, $3.75/million for 90 days. Each line of a build log counts as roughly one event. 

A 50 GB/month CI log ingestion at 30-day retention with say 60 million events (a fairly typical event-to-byte ratio for build logs) lands at $5 ingestion + $150 retention = $155/month. Bump retention to 90 days for forensics and the line becomes $5 + $225 = $230/month. The team that flipped on CI log shipping in a sprint without modelling retention costs is the team most likely to discover this surprise on the next bill. 

## Shipping logs to Splunk 

Splunk pricing is workload-based and quoted, but Splunk Cloud Standard is roughly $1.80-$2.00/GB/day of indexed data on average, billed monthly. A 50 GB/month ingestion (about 1.7 GB/day) at 30-day retention is roughly $90-110/month for the index plus storage. Splunk is the standard choice for enterprise security teams that already have a SIEM mandate, where the marginal cost of adding CI log sources is small relative to the existing Splunk investment. 

## Shipping logs to Grafana Loki (self-hosted) 

Loki is the self-hosted log store designed by Grafana Labs as a Prometheus-compatible alternative to Datadog and Splunk for log workflows. Cost structure is Kubernetes pods + S3 storage. A typical 3-node Loki deployment for 100 GB/day ingestion runs about $80-150/month total when you include S3 storage at $0.023/GB-month and the underlying compute. The big cost is engineering time: someone owns the Loki cluster, the dashboards, and the retention policies. 

The break-even versus Datadog at typical CI log volumes (50-100 GB/month) is about 6-12 months of operational time; for higher volumes (500 GB/month and up) Loki self-hosted typically lands an order of magnitude cheaper than the equivalent Datadog or Splunk bill, justifying the operational investment. 

## When to ship logs vs use the vendor default 

Three legitimate reasons to ship CI logs to an external store: cross-system correlation (you want to query CI logs alongside production logs in one tool when investigating an incident), retention beyond 90 days for compliance or forensics, and trend analysis on build duration / flake rates that the vendor UI does not surface well. 

If none of those apply, the vendor default is fine and the log retention line on your bill is zero. Many teams flip on log shipping out of platform-engineering enthusiasm and discover the storage bill is now $200/month for capability nobody uses. Audit annually: is anyone actually querying the CI logs in your observability tool? If not, switch off the export. 

## Frequently Asked Questions 
How long does GitHub Actions retain build logs? 
GitHub Actions retains build logs for 90 days by default on Free, Team, and Enterprise plans. Logs are stored at no charge as part of the platform; the 90-day retention is fixed and not configurable per workflow on the hosted plan. After 90 days the logs and the run metadata are purged. Teams that need longer retention typically ship logs to an external store (Datadog, Splunk, Loki) during the run. 
How big is a typical CI build log? 
Typical sizes per build, prose only with verbose flags off: Node/JavaScript 200-800 KB, Python 300 KB-1.2 MB, Java/Maven 500 KB-2 MB, Java/Gradle 800 KB-3 MB, Go 100-400 KB, .NET 400 KB-1.5 MB, Rust 600 KB-2 MB. Verbose flags (--verbose, -v, debug logs) typically inflate sizes 5-15x. Failed builds with stack traces are 2-3x normal. A team with 1,000 builds/day on a Java stack typically generates 1-3 GB/day of build logs. 
What does it cost to ship CI logs to Datadog? 
Datadog Logs Ingestion is $0.10/GB ingested, plus retention pricing: $1.27/million events for 7-day retention, $1.59/million for 15-day, $2.50/million for 30-day. A team generating 90 GB/month of CI logs (3 GB/day) at 30-day retention pays roughly $9 ingestion plus $50-150 retention depending on event count. Most teams discover this when their Datadog bill jumps after enabling CI log shipping. 
Is self-hosted log retention cheaper? 
Yes, by roughly 5-10x at scale. Loki on a 3-node Kubernetes deployment with S3 backing storage runs about $80-150/month for 100 GB/day ingestion at 30-day retention. ELK stack (Elasticsearch + Logstash + Kibana) self-hosted on 3 m5.large instances is roughly $250/month for similar volume but takes more engineering time to operate. The economics favour self-hosted above 50 GB/day; below that, vendor logging tools are operationally simpler and the price premium is small. 
Should I store CI logs at all beyond the vendor default? 
Three reasons to keep CI logs longer than 90 days: forensics on production incidents (need to trace which build introduced a regression), compliance requirements (SOC 2, ISO 27001 controls that mandate audit trails), and trend analysis (build duration regressions, flake rates, failure patterns over quarters). If none of those apply, the vendor default is fine and you can save 100% of the log retention cost line. 

## Related cost components 

- Artifact storage cost across vendors 
- Cross-region egress cost 
- Build minutes cost across vendors 
- CI runner cost: hosted vs self-hosted 
- Failed build rerun cost 
- 12 ways to cut your CI bill 
For vendor totals at your log volume, plug numbers into cicdcalculator.com . 

Updated 2026-08-04 


---
# PAGE: `cross-region-egress-cost` (https://cicdcost.com/cross-region-egress-cost)
CI/CD Cross-Region Egress Cost in 2026: When Your Pipeline Pays for Bandwidth | CICDCost.com Independent CI/CD cost research, read by teams pricing their pipeline. Sponsor this site → CICDCost.com is an independent comparison resource. Not affiliated with GitHub, GitLab, CircleCI, Buildkite, or any CI/CD vendor. Try our CI/CD calculator 
# CI/CD cross-region egress cost in 2026: where the bandwidth bill comes from 

Egress is the line item nobody plans for. CI pipelines pull container images, download dependencies, push artifacts, and ship logs. Each byte that crosses an AWS NAT, an Availability Zone boundary, or a region edge is metered. The rate per gigabyte is small. The volume is not. This page maps every per-byte fee a CI pipeline can incur in 2026, the path through your VPC where each fee actually lands, and the architecture changes that consistently cut the bill 70-95%. 

Headline at a glance (2026) 

Worst-case CI egress on AWS lands at $0.135/GB (egress + NAT processing) for traffic from a private subnet to the internet. Best-case is free using VPC endpoints to S3 and ECR. The 25x difference between worst and best is achievable with VPC endpoint configuration alone, no application changes. 

## AWS data transfer rates (2026) 

AWS prices data transfer in five separate dimensions: internet egress, cross-region transfer, cross-AZ within a region, NAT Gateway data processing, and Direct Connect / VPN data out. CI pipelines hit four of the five regularly. Numbers below per the AWS EC2 data transfer pricing page . 

| Path | Per-GB | Where it hits CI |
| Internet egress (first 10 TB/mo) | $0.09 | Pulling images from Docker Hub, GHCR, npm |
| Internet egress (next 40 TB) | $0.085 | Same, at higher volume |
| NAT Gateway data processing | $0.045 | Any traffic from private subnet to internet |
| Cross-AZ within region | $0.01 each direction | Multi-AZ runner pools talking to single-AZ services |
| Cross-region (us to eu) | $0.02 | Runners in one region pulling registry from another |
| VPC endpoint (S3, ECR) | $0.01 per endpoint-hour + $0.01/GB processed | Cheaper alternative to NAT for AWS-internal traffic |
| S3 to S3 same region | Free | Cache-to-runner if both are S3 in same region |
## The NAT Gateway trap 

Self-hosted CI runners typically live in private subnets for security reasons (no inbound from the internet, controlled outbound). Outbound traffic from a private subnet to the internet routes through a NAT Gateway. NAT Gateway charges two fees: an hourly charge ($0.045/hour per NAT, or about $33/month per NAT) and a per-GB processing charge ($0.045/GB on top of the underlying egress rate). 

A CI pipeline pulling 2 GB of dependencies and 1 GB of base images per build at 1,500 builds/month from a private subnet costs: 4,500 GB x ($0.09 internet egress + $0.045 NAT processing) = $607.50/month. Move the same workload to a public subnet (lose the private-subnet security model but gain direct internet routing): 4,500 GB x $0.09 = $405/month. Add VPC endpoints for ECR and S3 and route container pulls through them: the ECR portion (say 60% of volume) drops to $0.01/GB, total bill drops to roughly $270/month (2,700 GB x $0.01 + 1,800 GB x $0.135). 

The same architecture decision (private subnet + NAT vs public subnet vs VPC endpoints) makes a 2-3x difference in egress spend. Most teams do not revisit it after the initial cluster setup. 

### NAT Gateway data processing by region (2026) 

The $0.045/GB rate above is us-east-1. NAT Gateway pricing is regional: the per-hour charge and the per-GB data-processing charge always match each other within a region, but both rise outside US East. If your CI runners live in London (eu-west-2), Frankfurt (eu-central-1), or São Paulo (sa-east-1), the NatGateway-Bytes usage type on your bill is priced higher than the us-east-1 figure most guides quote. Rates below per the AWS NAT gateway pricing documentation , checked November 2026 against AWS's published price list. 

| Region | Per hour | Per GB processed |
| us-east-1, us-east-2, us-west-2 | $0.045 | $0.045 |
| us-west-1, eu-west-1 (Ireland) | $0.048 | $0.048 |
| eu-west-2 (London), eu-west-3 (Paris) | $0.050 | $0.050 |
| eu-central-1 (Frankfurt) | $0.052 | $0.052 |
| ap-south-1 (Mumbai) | $0.056 | $0.056 |
| ap-southeast-1 (Singapore) | $0.059 | $0.059 |
| ap-northeast-1 (Tokyo) | $0.062 | $0.062 |
| sa-east-1 (São Paulo) | $0.093 | $0.093 |
This is separate from, and on top of, the underlying data-transfer rate. A GB pulled from the internet through a NAT in eu-west-2 costs $0.050 (NAT processing) plus the standard egress rate, not $0.045. For a CI fleet moving several terabytes a month the region delta alone is a few hundred dollars a year. 

## Cross-region: the costly footgun 

The most expensive surprise on a CI bill is when the runners and the container registry are in different regions. ECR repositories are regional. If your team standardised on us-east-1 ECR and a new EU developer onboarded with runners in eu-west-2 to be near production, every container pull now crosses a region boundary at $0.02/GB inter-region plus the standard egress. 

Two fixes. The first is ECR Pull Through Cache, which automatically caches images from upstream registries into a regional ECR repository on first pull, with subsequent pulls served from the local region for free. The second is multi-region replication, which proactively replicates images to multiple regions on every push. The first is cheaper for sparsely-pulled images; the second is better for hot images. 

Same logic applies to package registries. Artifactory, Nexus, and JFrog all offer geo-distributed repositories where you push once and pull from the nearest mirror. These tools carry their own licence cost, so price the licence against your measured cross-region transfer before assuming it pays back. 

## The cache-as-egress-cut strategy 

Every byte you do not pull is a byte you do not pay egress on. CI caches (npm, pip, Maven, Cargo, Docker layer cache) reduce egress proportionally to cache hit rate. A 70% cache hit rate cuts dependency egress to 30% of cold-pull volume. 

Where the cache lives matters too. GitHub Actions cache lives on GitHub-managed S3, which means restoring a cache from a self-hosted runner crosses the internet (unless GitHub provides VPC peering, which they do not on standard plans). Restoring a 1 GB cache pulls 1 GB through your NAT. Self-hosted caches on local S3 or local block storage are essentially free per restore. The cost of running a cache server is offset by the egress savings within a few thousand builds. We cover the cache savings shape in detail on the build cache savings page . 

## GCP and Azure: similar shape, slightly different numbers 

GCP charges $0.12/GB for general internet egress (dropping with volume to $0.08 above 10 TB), with cross-region within North America at $0.02/GB and intercontinental at $0.08-0.12/GB. The full GCP network pricing page includes a detailed matrix. Cloud NAT charges similar to AWS NAT Gateway: $0.045/hour plus $0.045/GB processed. 

Azure egress out to the internet runs $0.087/GB for the first 5 GB free, dropping at higher volumes. Cross-region within Azure runs $0.02-0.05/GB depending on continents. Azure's NAT Gateway equivalent has comparable pricing. The architecture lessons (regional pinning, VPC/VNet endpoints, dependency mirrors) port across all three clouds with minor naming changes. 

## Audit pattern: find your CI egress today 

On AWS, look at the Cost Explorer filter Service = EC2-Other and the usage type DataTransfer-Out-Bytes , plus NatGateway-Bytes . Tag your CI runner instances with a consistent workload=ci tag and the report becomes per-team. Most teams discover their CI runners produce 30-50% of the engineering org's NAT Gateway processing cost: a single line item often $200-1,500/month. 

The fix is rarely a single change; it is a sequence: add VPC endpoints for ECR/S3 first (one config change, no application changes), then move package downloads to a local mirror (one infrastructure project), then revisit private-subnet vs public-subnet placement (a security review and a small refactor). Each step typically delivers 30-60% reduction. Done in sequence, three steps gets you to 90% reduction. 

## Frequently Asked Questions 
How much does AWS egress cost in 2026? 
AWS data transfer out to the internet is $0.09/GB for the first 10 TB, dropping to $0.085/GB beyond. Cross-region transfer is $0.02/GB inter-region. Cross-AZ within a region is $0.01/GB each direction (so $0.02/GB round-trip). NAT Gateway data processing is an additional $0.045/GB on top of egress for any traffic going through a NAT. CI runners in private subnets without VPC endpoints pay both egress AND NAT processing, which is the most common surprise on a CI bill. 
How much is NAT Gateway data processing in eu-west-2 (London)? 
In eu-west-2 (London), AWS NAT Gateway costs $0.050 per hour and $0.050 per GB of data processed (the NatGateway-Bytes usage type), checked November 2026 against AWS's published price list. That is higher than us-east-1, us-east-2, and us-west-2, which are $0.045/hour and $0.045/GB, and higher than eu-west-1 (Ireland) at $0.048. eu-west-2 matches eu-west-3 (Paris) at $0.050. NAT data processing is charged on top of the standard data-transfer rate, so a GB egressing to the internet through a NAT in London costs $0.050 plus the $0.09/GB internet egress. The per-hour and per-GB rates are always equal within a region. 
Why does my CI bill spike when I move runners to a new region? 
Because every dependency download, container image pull, and artifact upload now crosses a region boundary. If your container registry lives in us-east-1 and your runners moved to eu-west-2, every Docker pull is now $0.02/GB cross-region inter-cloud plus the destination egress. A pipeline pulling 2 GB of base images per build at 1,000 builds/month suddenly adds $40/month (2,000 GB x $0.02) for inter-region transfer alone. Mirror your registry per-region or pin runners and registry to the same region. 
Does GCP charge for cross-region egress like AWS? 
Yes, with similar pricing. GCP charges $0.12/GB for general internet egress, dropping with volume. Cross-region within North America is $0.02/GB, intercontinental is $0.08-0.12/GB. Within a region, traffic between zones is free for most traffic but Premium Network charges apply if you route through Cloud NAT or a load balancer. Azure pricing follows the same shape: internet egress $0.087/GB, cross-region $0.02-0.05/GB depending on continents. 
How do I cut CI egress cost? 
Five techniques. Use VPC endpoints for S3 and ECR so traffic stays on the AWS backbone (free, no NAT processing). Pull container base images from a region-local mirror or use ECR Pull Through Cache. Run npm/pip/Maven proxies (Artifactory, Nexus, or simple Harbor) inside your VPC so dependency downloads do not egress. Pin CI runners to the same region as your container registry and primary services. Use IPv6 routes where possible (some egress is cheaper or free over IPv6 on AWS). 
Are there CI vendors that bundle egress? 
Most hosted CI vendors include outbound bandwidth from the runner to your registry and to the internet up to a generous limit. GitHub Actions includes large egress allowances; CircleCI bundles network in their credit pricing; Buildkite hosted similarly. The egress problem appears when you self-host runners on AWS/GCP/Azure, because your CI is now paying the cloud provider rate for every byte that leaves the VPC. Self-hosting saves on compute but introduces a new line item that hosted CI customers never see. 

## Related cost components 

- Build cache savings: real numbers 
- Artifact storage cost across vendors 
- Log retention cost vendors hide 
- CI runner cost: hosted vs self-hosted 
- CI/CD caching strategies that cut minutes 
- Microservices CI cost 
Updated 2026-11-14 
