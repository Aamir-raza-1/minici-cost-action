# Verified Reference Content: 4 Calculation & Stack Intelligence Pillars

## comparison (/tmp/calc_comparison.html)

```text
CI/CD Platform Comparison 2026: Pricing, Features, and Total Cost | CICDCalculator.com
$
~/
cicdcalculator
[ menu ]
#
Independent CI/CD-cost research, read by platform and DevOps teams.
Sponsor this site →
CI/CD platform comparison 2026
./compare
--side-by-side
Six major CI/CD platforms, three team sizes, one honest comparison. We're not selling any of them, so the numbers and recommendations follow the use case rather than the highest-margin product.
Pricing matrix / per-minute Linux
Platform
Free tier
Seat
Linux rate
macOS
Self-host
Best at
GitHub Actions
2K mins / 500 MB
$4-21
$0.006/min
$0.062/min
Free
GitHub repos
GitLab CI
400 mins
$29+
$0.010/min
$0.06-0.12/min
Free
All-in-one stack
CircleCI
30K credits
$15
$0.006/min med
$0.12/min M4 Pro
Free
Flexible resources
Bitbucket Pipelines
3.5K mins
$8
$0.010/min
n/a hosted
Free
Atlassian shops
Buildkite
5 users / 10 jobs
$30
BYO compute
BYO compute
Native
High-volume / GPU
Jenkins
Unlimited (self-hosted)
$0
BYO compute
BYO Mac
Native
Compliance / scale
Total monthly cost / three team sizes
Numbers below assume Linux-only builds, 21 working days, average 8 minute build time, and consistent throughput. Self-hosted estimates include cloud compute and a typical operator-time allowance.
Platform
5 devs / 30 builds-day
20 devs / 120 builds-day
50 devs / 350 builds-day
GitHub Actions Team
~$40
~$255
~$905
GitLab Premium
~$145
~$580
~$1,450
CircleCI Performance
~$75
~$300
~$750
Bitbucket Premium
~$30
~$300
~$1,000
Buildkite Pro
~$165
~$860
~$2,150
Jenkins (self-hosted)
~$430
~$830
~$1,800
#
Green = best in row. Numbers are guides, not quotes; run your actual figures through the calculator.
Recommendations by use case
Solo dev / OSS
GitHub Actions Free
2,000 Linux mins is plenty for most personal projects. Free GitHub seats, no contract.
5-person startup, GitHub repos
GitHub Actions Team
Cheapest seat fee, included minutes cover early-stage volume, easiest setup.
Atlassian-stack team (Jira, Confluence)
Bitbucket Pipelines Premium
$8/user is hard to beat when you're already paying for Jira. Tight integration.
Mid-size product team, polyglot stack
GitLab Premium
10K included mins per user often makes it cheaper than rivals once usage is real. Source control + CI/CD in one.
iOS / macOS heavy team
Self-hosted Buildkite or CircleCI
Owning the Mac fleet beats per-minute macOS rates fast. Buildkite's BYO model fits naturally.
Regulated / on-prem only
Self-hosted Jenkins or GitLab CE
Full control of where code runs and where artifacts live. Higher operator cost in exchange.
High-volume (>50K mins / mo)
Buildkite Pro or self-hosted
Per-minute pricing breaks down at this scale. Per-user models or self-hosted infra win.
Enterprise (200+ devs)
Custom contract with anyone
Prepaid minute pools, multi-year discounts, and SAML SSO all on the table. Don't pay list.
Run your own numbers
The interactive calculator on the homepage takes your team size, builds per day, average build minutes, parallelism, and runner OS, and returns a side-by-side cost across all six platforms in real time.
>
run the calculator
cost optimization
migration costs
Frequently Asked Questions
#
click any question to expand
Which CI/CD platform is cheapest for a 5-person team?
>
GitHub Actions Team plan at around $20-100/month is usually cheapest for 5-person teams. The free 3,000 Linux minutes per month covers most small-team workloads, and the seat fee is just $4 per user. Bitbucket Pipelines Premium ($8/user) is comparable for Atlassian-stack shops. Skip GitLab Premium and CircleCI Performance unless you have specific feature needs at this scale.
Which is cheapest for 50+ developers?
>
It depends on the workload mix. On pure Linux volume, GitHub Actions Team stays cheapest even at 50+ devs since the January 2026 rate cut (around $780 at 100K monthly minutes). Buildkite at $30 per user with your own compute lands around $2,000/month all-in for 50 devs, and wins once macOS fleets, GPU runners, or very high build volume enter the picture, because those are exactly where hosted per-minute pricing compounds fastest.
What's the most full-featured CI/CD platform?
>
GitLab Ultimate is the most feature-complete out of the box: SAST, DAST, dependency scanning, license compliance, value-stream analytics, and merge approvals all built in. The trade-off is the per-user price. GitHub Actions plus GitHub Advanced Security gets you to feature parity but Advanced Security is sold separately at significant cost.
Which platform has the best macOS support?
>
Hosted macOS pricing varies more than Linux: GitHub Actions charges $0.062/min since the January 2026 cuts, while CircleCI's M4 Pro Medium runs $0.12/min (200 credits). For self-hosted Mac fleets, Buildkite is the most popular choice because the agent model fits running a Mac mini cluster naturally. CircleCI has decent self-hosted Mac support too. Avoid Bitbucket Pipelines for iOS-heavy work; it doesn't offer native macOS runners.
Should I prefer hosted or self-hosted?
>
Hosted CI/CD is the right default for teams under 20 developers because the engineering time to operate self-hosted exceeds the per-minute fees. Above 50 developers or roughly 100,000 monthly build minutes (a higher bar since GitHub's January 2026 rate cut), self-hosted (or a hybrid model) usually wins on cost. Between 20 and 50 developers it's situational. Hybrid (hosted for PR checks, self-hosted for big nightly jobs) is often the sweet spot.
How does enterprise pricing actually work?
>
All major CI/CD platforms negotiate at scale. Typical enterprise discounts range from 10-30% off list price at six-figure ARR commitments. Volume seats, multi-year terms, and prepaid minute pools are the most common levers. Free trial credits and dedicated solution-architect access are standard at these scales. Don't pay list price if you're spending more than $50K/year.
```

## optimization (/tmp/calc_optimization.html)

```text
How to Reduce CI/CD Costs in 2026: 10 Proven Strategies | CICDCalculator.com
$
~/
cicdcalculator
[ menu ]
#
Independent CI/CD-cost research, read by platform and DevOps teams.
Sponsor this site →
How to reduce CI/CD costs
./optimize
--proven-strategies
Ten optimisation techniques that consistently cut CI/CD spend, ranked by impact-to-effort. Each comes with a typical savings range and a one-paragraph implementation note. Pick the top two or three for your stack and you'll usually halve your monthly CI bill in a sprint.
01
Cache dependencies aggressively
saves
20-50%
Most pipelines spend 40-70% of build time on npm install / pip install / gradle build. Cache the dependency directory and the lock file together. Restore on cache hit, install only on miss. Every major CI/CD platform has a built-in cache action. This is the highest-impact single change you can make.
02
Skip jobs when nothing changed
saves
20-40%
Use path filters (paths: in GitHub Actions, rules: changes: in GitLab CI) so backend-only commits don't trigger frontend tests, and docs commits don't trigger anything. On busy monorepos this single change often pays for the entire optimisation effort.
03
Run heavy jobs only on protected branches
saves
10-30%
Full integration tests, performance benchmarks, and Cypress runs don't need to fire on every PR commit. Restrict them to main, release branches, or scheduled cron jobs. PR checks become fast lint + unit tests; deeper runs happen on merge.
04
Use spot / preemptible instances for self-hosted runners
saves
60-80% on compute
Spot instances on AWS or preemptible VMs on GCP cost 60-80% less than on-demand. CI jobs are idempotent so reclamation is harmless: the agent dies, the orchestrator retries on a fresh instance. Just don't put long-running deploy steps on spot.
05
Right-size your runners
saves
15-30%
A 4-core runner finishing in 4 minutes can be cheaper than a 2-core finishing in 12. Time the trade-off, don't assume the cheap runner wins. CircleCI's resource_class, GitHub Actions' larger_runner labels, and GitLab CI's tags: all let you target specific machine sizes per job.
06
Split slow test suites across parallel runners
saves
wall-clock 50-80%, billable similar
knapsack_pro, ts-jest --shard, pytest-split, CircleCI --split-by=timings: all let you carve a 30-minute test run into ten 3-minute parallel jobs. Total billable minutes stay similar but PR feedback latency drops dramatically. Faster feedback means fewer retries means fewer billable minutes overall.
07
Set artifact retention to days, not months
saves
$5-50/month per repo
Default retention on GitHub Actions is 90 days. Most artifacts are useful for 7-14. Drop retention with retention-days: 7 in your workflow yaml. Multiply across repos and you're saving real storage cost, especially on the Free and Team plans where storage past the included quota is billed at $0.25/GB/month.
08
Reduce Docker image rebuild work
saves
10-25%
Use multi-stage Dockerfiles with deterministic layer ordering, push intermediate stages as cache to a registry, and pull them on subsequent builds. Tools like docker buildx with type=registry caching make this almost automatic. A 6-minute image rebuild becomes 30 seconds when layers are cached properly.
09
Move artefacts off platform-billed storage
saves
$10-100/month
Most CI/CD platforms charge premium rates for artifact storage. Push large artefacts (tarballs, Docker images, video recordings of e2e runs) to S3 or GCS instead. Standard cloud storage is 10-100x cheaper than CI/CD platform-included storage past the free tier.
10
Set hard spending limits
saves
the runaway-loop bill
Every platform supports a spending cap. Set one. A single misconfigured workflow that triggers on every push of every branch can rack up four-figure bills overnight. Spending limits won't stop bad workflows but they'll stop the invoice. GitHub Actions, GitLab CI, and CircleCI all support hard caps at the org level.
Suggested order of work
01.
Set spending limits today. Five-minute job, prevents disaster.
02.
Add caching to your slowest install step. Ship in a day.
03.
Add path filters to skip irrelevant jobs. Half a day on a monorepo.
04.
Move artifact retention from default to 7-14 days. Five-minute change.
05.
Move heavy jobs off PR triggers. Few hours, big PR latency win.
06.
Right-size runners using actual job timings. One-week measurement.
07.
Evaluate self-hosted runners for hot paths. Only if you're past 50K minutes / month.
Related guides
cost calculator
platform comparison
self-hosted vs cloud
migration guide
platform engineering cost
Frequently Asked Questions
#
click any question to expand
What's the single biggest CI/CD cost saving?
>
Aggressive dependency caching. Most pipelines spend 40-70% of their time installing npm, pip, gradle, or apt packages. Properly configured caches drop that to seconds and cut billable build minutes by a third or more. Every major CI/CD platform has a built-in cache action; the impact-to-effort ratio is unbeatable.
Does test parallelism save money or cost more?
>
It depends on the platform. On per-minute platforms (GitHub Actions, GitLab CI), parallelism cuts wall-clock time but doesn't change billable minutes; you're still paying for the same total compute. On credit-based platforms (CircleCI), parallelism is free if you stay within concurrency limits. Parallelism saves money when it allows you to use smaller machines that finish faster overall.
When should I move to self-hosted runners?
>
When you cross roughly 80,000 build minutes per month consistently (a higher bar since GitHub's January 2026 rate cut) and have an engineer who can spend 4-6 hours per month on operator tasks. Below that threshold the engineering time costs more than the per-minute fees. Hybrid setups (hosted for PR checks, self-hosted for big nightly jobs) often beat pure-cloud or pure-self-hosted on cost.
How much do spot instances save?
>
AWS Spot, Google preemptible, and Azure Spot VMs typically run 60-80% cheaper than on-demand. The trade-off is they can be reclaimed at any time. CI/CD jobs are an excellent fit because they're idempotent: a reclaimed agent retries on a new instance, costing one extra minute of compute. Pair with autoscaling to drop costs further during off-hours.
What's path-based filtering and is it worth it?
>
Path-based filtering tells your CI to skip jobs when only certain files change. Backend-only PR? Skip the slow Cypress tests. Docs typo? Skip the entire pipeline. Worth a few hours of yaml work because it can cut billable minutes by 20-40% on a busy monorepo. Both GitHub Actions (paths:) and GitLab CI (rules: changes:) support it natively.
Should I split tests across runners?
>
Yes if your test suite takes more than 10 minutes. Test splitting (using tools like ts-jest --shard, knapsack_pro, or CircleCI's --split-by=timings) divides tests across N parallel jobs. Wall-clock time drops to 1/N. On per-minute platforms total billable minutes stay roughly the same, but PR feedback latency goes from 30 minutes to 5, which is a real productivity win.
```

## larger_runners (/tmp/calc_larger_runners.html)

```text
GitHub Actions Larger Runners Pricing 2026: Per-Minute Rates | CICDCalculator.com
$
~/
cicdcalculator
[ menu ]
#
Independent CI/CD-cost research, read by platform and DevOps teams.
Sponsor this site →
GitHub Actions larger runners pricing
./per-minute
--always-billed --2026
By
Oliver Wakefield-Smith
, Founder, Digital Signet
The standard GitHub-hosted runners (Linux 2-core, Windows 2-core, macOS) are what your free minutes and plan allowances buy you. Larger runners are a separate SKU: more cores, more RAM, optional GPUs and static IPs, priced per minute and billed from the first second. The catch that surprises most teams is that larger runners never touch your included minutes, so a job you thought was covered by the free tier can quietly meter a bill. This page lists every current per-minute rate and shows what a real month of larger-runner CI costs.
Direct answer
Larger runners are billed per minute and are always charged for
— even on public repositories, even when you have free minutes left. Linux x64 runs from
$0.012/min (4-core)
to
$0.162/min (64-core)
. Windows larger runners cost roughly double. arm64 runners are the cheapest at every size, and a 2-core arm64 runner ($0.005/min) undercuts even the standard Linux rate.
Linux 4-core (x64)
$0.012 / min
Linux 64-core (x64)
$0.162 / min
Cheapest (arm64 2-core)
$0.005 / min
Draws free minutes?
Never
Linux x64 larger runners
The default larger-runner family. Every rate is all-in and billed per minute of execution time.
Runner
Rate / min
Cost / hour
10,000 min / mo
Linux 4-core
$0.012
$0.72
$120
Linux 8-core
$0.022
$1.32
$220
Linux 16-core
$0.042
$2.52
$420
Linux 32-core
$0.082
$4.92
$820
Linux 64-core
$0.162
$9.72
$1,620
Rates per docs.github.com (Actions runner pricing reference), verified August 2026. Cost/hour and monthly columns are this page's own arithmetic (rate × minutes).
Windows x64 larger runners
Windows larger runners cost close to double the Linux rate for the same core count, reflecting Windows licensing on top of the compute.
Runner
Rate / min
Cost / hour
10,000 min / mo
Windows 4-core
$0.022
$1.32
$220
Windows 8-core
$0.042
$2.52
$420
Windows 16-core
$0.082
$4.92
$820
Windows 32-core
$0.162
$9.72
$1,620
Windows 64-core
$0.322
$19.32
$3,220
Rates per docs.github.com Actions runner pricing reference, verified August 2026.
arm64 larger runners: the cheapest option
Linux arm64 larger runners undercut their x64 equivalents at every size. If your workload already runs on ARM, or can be made architecture-agnostic, this is the single biggest lever on a larger-runner bill: roughly 40 percent off the x64 rate.
Runner (arm64)
Rate / min
x64 equivalent
Saving
arm64 2-core
$0.005
n/a (below std)
cheapest of all
arm64 4-core
$0.008
$0.012
-33%
arm64 8-core
$0.014
$0.022
-36%
arm64 16-core
$0.026
$0.042
-38%
arm64 32-core
$0.050
$0.082
-39%
arm64 64-core
$0.098
$0.162
-40%
Rates per docs.github.com Actions runner pricing reference, verified August 2026. Saving column is this page's own arithmetic against the x64 rate.
GPU runners
GitHub also hosts GPU larger runners for machine-learning builds, model tests and CUDA workloads. Like all larger runners, they are billed per minute and never draw from included minutes.
GPU runner
Rate / min
Cost / hour
Linux 4-core GPU
$0.052
$3.12
Windows 4-core GPU
$0.102
$6.12
Rates per docs.github.com Actions runner pricing reference, verified August 2026.
The catch that bites
Larger runners never use your free minutes
Standard runners spend your included allowance first (2,000 Linux minutes on Free, 3,000 on Team, 50,000 on Enterprise) and only meter once that is gone. Larger runners work differently: GitHub charges for every minute they execute,
even on public repositories and even when your plan's included minutes are untouched
. Move a routine job onto a larger runner to shave a few minutes and you swap a free build for a metered one. Keep the everyday PR checks on standard runners and reserve larger runners for the jobs that genuinely need the cores, the RAM, the static IP or the GPU.
Does a bigger runner actually cost more?
Not always. A larger runner that finishes the same build in proportionally less time can cost the same or less, and returns the result sooner. The honest comparison is total cost per build, not the per-minute rate.
Linux 2-core (standard)
$0.180
30 min build
$0.006/min x 30. Cheapest per minute, slowest result.
Linux 8-core
$0.176
8 min build
$0.022/min x 8. Same cost, 22 minutes sooner.
Linux 16-core
$0.168
4 min build
$0.042/min x 4. Cheapest overall and fastest, if the build parallelises.
The catch is that the speed-up only materialises if the build actually uses the extra cores. A single-threaded compile or a test suite that runs serially will take the same wall-clock time on a 16-core runner as on a 2-core one, and you will pay seven times as much per minute for nothing. Measure your build's parallelism before upgrading.
Keep going
full GitHub Actions pricing
is it free?
cost calculator
2026 rate changes
cut your bill
back to calculator
Frequently Asked Questions
#
click any question to expand
How much do GitHub Actions larger runners cost?
>
GitHub-hosted larger runners are billed per minute at a rate that scales with core count. On Linux x64 the rates are $0.012 per minute for 4-core, $0.022 for 8-core, $0.042 for 16-core, $0.082 for 32-core and $0.162 for 64-core. Windows larger runners cost roughly double: $0.022 (4-core) up to $0.322 (64-core). arm64 larger runners are cheaper than x64 at every size. Rates per the docs.github.com Actions runner pricing reference, verified August 2026.
Do larger runners use my free or included minutes?
>
No. This is the single most important thing to understand about larger runners: they are always charged for, even on public repositories or when you still have included minutes left on your plan. Only the standard runners (Linux 2-core, Windows 2-core, macOS) draw from the free 2,000-minute Free-plan allowance or your Team/Enterprise pool. The moment a job runs on a larger runner, it is metered per minute from the first second.
Are arm64 larger runners cheaper than x64?
>
Yes, noticeably. GitHub prices Linux arm64 larger runners below their x64 equivalents at every size: $0.008 per minute for 4-core (vs $0.012 x64), $0.014 for 8-core (vs $0.022), $0.026 for 16-core (vs $0.042), $0.050 for 32-core (vs $0.082) and $0.098 for 64-core (vs $0.162). There is even a 2-core arm64 larger runner at $0.005 per minute, cheaper than the standard Linux 2-core rate of $0.006. If your build runs on ARM (or can be made architecture-agnostic), arm64 runners cut the compute bill by roughly 40 percent.
Do larger runners cost extra on public repositories?
>
Yes. Standard runners are free and unlimited on public repositories, but larger runners are an exception: they are billed per minute even for public repos. If you attach a larger runner to an open-source project's workflow, you pay the per-minute rate for every minute it executes, so most public projects keep heavy jobs on the free standard runners and reserve larger runners for private, time-critical builds.
What is the cheapest GitHub Actions larger runner?
>
The Linux arm64 2-core larger runner at $0.005 per minute is the cheapest option and undercuts even the standard Linux 2-core rate ($0.006). Among the higher-core options, arm64 is always cheaper than x64 for the same core count. If you only need extra RAM or a static IP (a common reason to reach for larger runners) rather than raw cores, the smallest arm64 runner is the most economical way to get it.
How much do GPU runners cost on GitHub Actions?
>
GitHub-hosted GPU larger runners are billed per minute like other larger runners: a Linux 4-core GPU runner is $0.052 per minute and a Windows 4-core GPU runner is $0.102 per minute. As with all larger runners, GPU runners are always billed and never draw from included minutes, so a single hour of GPU CI time on Linux costs about $3.12.
When are larger runners worth the extra cost?
>
Larger runners pay off when a faster machine finishes the job in enough less wall-clock time to offset the higher per-minute rate, or when a build genuinely needs more RAM, more cores for parallel test suites, a static outbound IP for allow-listing, or a GPU. A 16-core runner at $0.042/min that completes a build in 4 minutes ($0.168) beats a 2-core runner at $0.006/min that takes 30 minutes ($0.180) and returns the result 26 minutes sooner. Always time the trade-off on your real workload rather than assuming the cheapest per-minute runner wins.
```

## ios (/tmp/calc_ios.html)

```text
iOS CI/CD Cost 2026: macOS Runners, Build Times, Real Bills | CICDCalculator.com
$
~/
cicdcalculator
[ menu ]
#
Independent CI/CD-cost research, read by platform and DevOps teams.
Sponsor this site →
iOS CI/CD cost in 2026
./ios-ci
--macos-10x-linux --self-host-or-cry
iOS CI is the most expensive line item in most mobile-shipping engineering organisations. Apple's licensing rules require every macOS workload to run on real Mac hardware. The cloud CI/CD industry has built around this constraint, but the per-minute pricing remains roughly 10x the equivalent Linux rate on every major platform. For a team shipping iOS at any meaningful volume, the platform decision is dominated by macOS economics. This page works through what you actually pay in 2026, what changes with self-hosted Mac fleets, and which CI/CD platforms make iOS bearable.
All numbers below come from each vendor's public pricing page, re-verified 15 September 2026. Mac mini hardware costs come from
Apple's Mac mini configurator
, where the range currently starts at $899.
Hosted macOS pricing across platforms
A per-minute rate on its own is a bad way to compare iOS CI, because two of these platforms do not actually sell minutes. The right-hand column is what bites once you commit: seat caps, machine availability, build-count ladders and concurrency.
Platform
Mac class
$ / min
10K mins / mo
What limits it
GitHub Actions
macOS 3/4-core
$0.062
$620
Pure metered. Included minutes (2,000 Free, 3,000 Team) drain at the macOS rate
CircleCI
Mac M4 Pro Medium
$0.120 (200 credits)
$1,200
macOS concurrency 1 on Free, 15 on Performance
CircleCI
Mac M4 Pro Large
$0.240 (400 credits)
$2,400
As above. Credits are $15 per 25,000
Bitrise
Pro
macOS Large (Apple Silicon)
$0.0096 + plan
$437 plan + minutes
Pro is a build-count ladder, not a flat fee. 10K mins of 20-min builds is 500 builds, the $437 rung. Overage at an unpublished premium rate
Bitrise
Starter
macOS Medium only
Minutes unmetered
$99-$299 plan
1-4 concurrent builds by rung, members unlimited.
macOS Large unavailable. Recommended up to 100 builds/mo
Codemagic
PAYG
Mac mini M2
$0.095
$902.50
First 500 M2 mins/mo free. No seat or plan fee. 120-min build cap
Codemagic
PAYG
Mac mini M4
$0.114
$1,140
No free allowance on M4. Extra concurrency $49/mo each
Codemagic
annual M2
Mac mini M2
Flat, unmetered
$332.50
$3,990/yr. 3 concurrent builds; more at $1,500/yr each
Buildkite
+ self-hosted Mac mini
Mac mini you own
~$0.003 amortised
~$25 hardware + plan
Plan is per seat.
Free covers 5 users and 10 concurrent jobs; Pro is $30 per active user/mo
#
Rates from
docs.github.com
,
circleci.com/pricing
,
bitrise.io/pricing
,
codemagic.io/pricing
and
buildkite.com/pricing
, all re-checked 15 September 2026. The 10K column is our arithmetic over those published rates. Bitrise's Pro build-volume rungs above the $218 entry price are not published by Bitrise; they were confirmed to us by Bitrise in August 2026. Self-hosted Mac mini "amortised" is hardware at Apple's current $899 entry price over 36 months plus electricity, and excludes the CI control-plane fee, which is the column to its right.
Real monthly cost for an iOS team
Three team scenarios, all running 20-minute iOS builds on Apple silicon macOS or equivalent self-hosted Mac mini hardware. Every line is the published rate times the stated inputs, shown so you can check it. Build count matters as much as minutes here, because two of the five platforms price on builds or seats rather than time.
5-dev iOS team
6,300 macOS mins / mo
15 builds/day × 21 days = 315 builds × 20 mins
GitHub Actions:
$390.60
(6,300 × $0.062)
CircleCI M4 Pro Medium:
$756
(6,300 × $0.120)
Codemagic PAYG M2:
$551
((6,300 − 500) × $0.095)
Codemagic annual M2:
$332.50
($3,990 / 12, unmetered)
Bitrise Pro:
$437+
(315 builds = the 500-build rung, plus minutes)
Buildkite Free + 1 Mac mini:
~$25
($0 plan at 5 users + $899/36)
15-dev iOS team
12,600 macOS mins / mo
30 builds/day × 21 days = 630 builds × 20 mins
GitHub Actions:
$781.20
(12,600 × $0.062)
CircleCI M4 Pro Medium:
$1,512
(12,600 × $0.120)
Codemagic PAYG M2:
$1,149.50
((12,600 − 500) × $0.095)
Codemagic annual M2:
$332.50
($3,990 / 12, unmetered)
Bitrise Pro:
$655+
(630 builds = the 750-build rung, plus minutes)
Buildkite Pro + 2 Mac minis:
~$500
(15 × $30 = $450, + $50 hardware)
30-dev iOS team
25,200 macOS mins / mo
60 builds/day × 21 days = 1,260 builds × 20 mins
GitHub Actions:
$1,562.40
(25,200 × $0.062)
CircleCI M4 Pro Medium:
$3,024
(25,200 × $0.120)
Codemagic PAYG M2:
$2,346.50
((25,200 − 500) × $0.095)
Codemagic annual M2:
$332.50
($3,990 / 12, 3 concurrent slots)
Bitrise Pro:
$1,310+
(1,260 builds = the 1,500-build rung, plus minutes)
Buildkite Pro + 4 Mac minis:
~$1,000
(30 × $30 = $900, + $100 hardware)
#
Two corrections worth spelling out, because earlier versions of this page got both wrong.
Bitrise
is not a flat plan fee plus cheap minutes: Pro is a ladder priced by monthly build count, so the figure shown is the rung that covers the build count, and machine minutes bill on top against the tier's allowance, with anything beyond it at a premium Pay-As-You-Go rate Bitrise does not publish. Read every Bitrise number here as a floor.
Self-hosting
is not free either: the Mac hardware is cheap, but the CI control plane is per seat, and at 15 and 30 developers Buildkite's $30 per active user dominates the hardware cost completely. Codemagic's flat annual plan carries 3 concurrent build slots, which covers 25,200 monthly minutes spread across a working day but not a narrow release window; extra slots are $1,500 a year.
The Mac mini fleet: how it actually works
A self-hosted Mac mini fleet for iOS CI is the single most leveraged cost decision a mobile team can make. The setup is straightforward and the savings are dramatic.
Buy one to six Mac minis, depending on how much concurrent build capacity you need. Apple's range currently starts at $899 and runs to $6,799; a mini at $899-1,400 amortises to $25-39 per month over 36 months. Install macOS, Xcode, Fastlane and your CI agent of choice (Buildkite agent, GitHub Actions self-hosted runner, GitLab Runner, all work fine). Connect the minis to your office network or a colocation facility. Point your CI pipeline at the self-hosted runner pool and your existing iOS pipelines just run on them.
One Mac mini sustains roughly 8,000-10,000 monthly build minutes at full continuous load with adequate cooling, which is our own operating estimate rather than a vendor figure. So a typical 15-developer iOS team running 12,600 monthly minutes needs two minis with margin, three for comfort. Three minis amortised plus electricity is around $85-95 monthly. The same compute on hosted macOS would cost $780 on GitHub Actions or $1,512 on CircleCI. What that comparison leaves out is the control plane: self-hosted agents still need a CI platform, and Buildkite charges $30 per active user per month above its five-user free plan, so the honest all-in for that 15-developer team is nearer $500 than $95.
Operational overhead is small. Each mini needs occasional macOS updates (handled gracefully but they require physical interaction with the device, so plan for a 30-minute window twice a year per mini), occasional Xcode upgrades when Apple ships a new version, and a plan for what happens if one fails. Most teams keep a hot spare or accept that one mini being offline for a week is a tolerable degradation.
Hosted alternatives if self-hosted is not an option
Some teams cannot self-host Mac hardware. Compliance requirements, lack of office space, distributed teams without a central location, or just unwillingness to take on physical hardware ownership. The colocation route via MacStadium ($80-200 per fully managed Mac monthly) sits between self-hosted and hosted CI; cost is meaningfully lower than per-minute hosted CI but operator complexity is also lower than rolling your own.
For pure-hosted, Bitrise's published pay-per-minute macOS rate ($0.0096 per minute on macOS Large, pass-through with no infrastructure markup) sits far below GitHub Actions hosted macOS at $0.062 (the all-in rate after the January 2026 cut). That rate is real, but it is not the bill. Bitrise prices the plan itself by monthly build count, and the macOS Large machine is only available on Pro and above, so the cheap rate and the cheap plan cannot be combined. Price Bitrise by counting your workflow runs, then find the rung. CircleCI's M4 Pro classes start at $0.12 per minute, the priciest per-minute hosted macOS, though the M4 Pro hardware is meaningfully faster per build, which claws back part of the gap on wall-clock-bound pipelines.
Bitrise specifically is worth considering for mobile-first teams because the platform also handles code signing, simulator setup, store deployment and crash analytics natively. The saved pipeline-assembly time, not the per-minute rate, is the real argument for it.
Codemagic
is the other mobile-first option and the one with the simplest pricing on this page. Pay-as-you-go meters a Mac mini M2 at $0.095 a minute with no plan fee, no seat fee and the first 500 M2 minutes each month free, which makes it the cheapest way to run a low, bursty iOS pipeline without committing to anything. Its flat annual plans are the interesting part at volume: $3,990 a year buys unlimited Mac mini M2 minutes and three concurrent builds, which works out at $332.50 a month and stops being a per-minute question entirely. Above roughly 4,000 macOS minutes a month that flat plan is the cheapest hosted macOS on this page by a wide margin, and it stays flat as your build volume climbs, which is the opposite of how Bitrise and the metered platforms behave. The constraint to check is concurrency rather than minutes: three slots, with more at $1,500 a year each, and a 120-minute cap on any single build.
Recommendation by team shape
Solo or 2-3 person iOS team on a tight budget: GitHub Actions with the public-repo unlimited macOS for OSS, Codemagic's free tier (500 macOS M2 minutes a month, single user) or its pay-as-you-go with no plan fee, Bitrise's free Hobby tier for evaluation, or the Buildkite Free plan (five users, ten concurrent jobs) with a single self-hosted Mac mini.
5-15 person mobile-first team without dedicated platform engineering: Bitrise or Codemagic, and the choice turns on your build shape. Bitrise for the deeper mobile workflow library, priced on the Pro rung your monthly build count actually lands on rather than the advertised entry price. Codemagic if you would rather buy minutes than builds, where the $3,990-a-year flat plan makes the bill predictable no matter how the build count moves. Plan to revisit when the team passes 20 developers.
10-30 person mobile team with a platform engineer: Buildkite or GitHub Actions self-hosted with a 2-4 Mac mini fleet, but do the seat arithmetic before assuming it wins. The compute saving is overwhelming; the per-seat control-plane fee is not, and at 30 developers Buildkite Pro alone is $900 a month, which is more than Codemagic's flat annual plan costs for the whole workload. Self-hosting wins clearly when you have build volume that a flat plan's concurrency cannot absorb, or compliance rules that rule hosted macOS out.
30+ person mobile team or strict compliance requirements: self-hosted Mac fleet at scale via Buildkite, or MacStadium colocation with the CI/CD platform of your choice. The Mac fleet pays back its capex inside one quarter at any meaningful volume.
Related deep dives
Bitrise pricing
Codemagic pricing
Buildkite agent cost
Android CI cost
Buildkite vs Actions
self-hosted vs cloud
cost calculator
Frequently Asked Questions
#
click any question to expand
Why does iOS CI cost so much?
>
Apple's licensing forces all macOS hosting to use real Mac hardware. Mac hardware is more expensive than commodity Linux servers and harder to virtualise efficiently. Every cloud-CI provider that offers macOS runners has to either own Mac hardware (MacStadium, GitHub Actions) or rent it (CircleCI, GitLab) and the cost passes through to per-minute pricing. The result is roughly 10x the per-minute cost of Linux runners on every major platform.
What does a typical iOS pipeline cost monthly?
>
A 10-developer iOS team running 30 daily iOS builds at 20 minutes each consumes 12,600 monthly macOS minutes across 630 builds. On GitHub Actions hosted macOS at $0.062 per minute (the all-in rate since January 2026), that is $781.20. On CircleCI Mac M4 Pro Medium (200 credits per minute at $0.0006), that is $1,512. On Codemagic it is $1,149.50 on pay-as-you-go after the 500 free M2 minutes, or $332.50 a month on the $3,990-a-year flat plan. Bitrise is the one that cannot be quoted as a per-minute figure: Pro is priced by monthly build count, and 630 builds lands on the $655 rung, with machine minutes billing on top of the tier allowance and anything past it at an unpublished premium rate. Self-hosting is cheap on hardware and not free overall: three Mac minis amortise to about $75 a month, but the CI control plane on top is per seat, so Buildkite Pro for ten developers adds $300.
How much can I save with a Mac mini fleet?
>
On the compute line, roughly 80-95 percent at moderate to high volume. Apple's Mac mini range currently starts at $899; amortised over 36 months plus electricity that is about $28-32 per Mac per month, so three minis run at $85-95. The same throughput on hosted macOS bills $200-500 monthly per equivalent slot. The saving is real but it is not the whole bill: you still pay a CI control plane on top, and those are priced per seat, so a 15-developer team on Buildkite Pro adds $450 a month regardless of how cheap the hardware is. The Mac mini maths is overwhelming for the compute and roughly neutral once seats are counted, which is why it wins hardest for small teams with heavy build volume.
Where do I host self-hosted Mac minis?
>
Three common options. (1) In your office on a desk or in a small server cabinet, connected to your office network. Cheapest, requires minimal physical security. (2) At a colocation facility specialising in Mac hardware, such as MacStadium, typically $80-200 per Mac monthly fully managed (their published range; check current rates). Middle ground: not as cheap as office but no physical responsibility. (3) Cloud-rented Mac instances from MacinCloud, MacStadium Cloud, AWS EC2 Mac. Most expensive of the three, justified only if you have specific compliance requirements that prevent self-managed hardware.
Is Bitrise worth its premium?
>
For 5-15 person mobile-first teams without dedicated platform engineers, often yes, but price it on build count rather than on the headline. Bitrise's mobile-specific workflow library (code signing, profile rotation, simulator selection, App Store deploy) saves substantial pipeline-assembly time versus building the equivalent on a general-purpose platform. The trap is the entry price: Starter advertises from $89 a month, but that bottom rung carri
```

## android (/tmp/calc_android.html)

```text
Android CI/CD Cost 2026: Emulator-Heavy Pipeline Pricing | CICDCalculator.com
$
~/
cicdcalculator
[ menu ]
#
Independent CI/CD-cost research, read by platform and DevOps teams.
Sponsor this site →
Android CI/CD cost in 2026
./android-ci
--gradle --emulator-heavy
Android CI is more expensive than backend CI because Android pipelines are bigger and slower than they appear. Gradle initialisation takes seconds, dependency resolution takes minutes, and emulator-based instrumentation tests turn what looks like a quick check into a sustained CI workload. Unlike iOS, Android does not face the macOS pricing penalty, so the platform choice is dominated by Linux runner cost and emulator handling. This page works through what every major CI/CD platform actually charges for an Android pipeline in 2026 and where the optimisation wins are.
Pricing references each vendor's public pricing page, re-verified August 2026; the Bitrise Pro build-tier ladder is confirmed by Bitrise (Aug 2026).
Gradle build cache documentation
for cache configuration, and the
Android emulator runner action
for emulator integration on Actions.
Typical Android pipeline composition
A representative Android pipeline for a mid-size production app runs roughly:
# Per PR push:
1. Gradle init + dependency resolution:
2-4 min
2. Lint + ktlint + detekt:
1-2 min
3. Unit tests:
2-4 min
4. Build debug APK:
1-2 min
Total per PR:
~8-10 min
# On merge to main:
5. Instrumentation tests on emulator:
5-15 min
6. Build release APK + sign:
3-5 min
Total on merge:
~20-30 min
For a 10-developer team pushing 30 PRs daily plus 5 merges, monthly Linux minute consumption is around 6,300 (PR builds) plus 3,000 (merges plus nightly) for a total of about 9,300 monthly Linux minutes. Larger Android shops with extensive instrumentation tests can easily double or triple that.
Per-platform monthly cost for Android
Numbers below are for a 10-developer team consuming 9,300 monthly Linux minutes (mix of PR plus merge plus nightly).
Platform
Plan
Seats
Compute
Total / mo
GitHub Actions
Team
$40
$38 (6.3K over 3K @ $0.006)
$78
GitLab CI
Premium
$290
$0 (in 10K free)
$290
CircleCI
Performance
$90 (user charges)
$45 (63K credits over 30K)
$135
Bitrise
Pro (750 builds)
$655 (flat plan)
$0 (builds in allowance)
$655
Buildkite
Pro
$300 (10 x $30)
$60 (2 EC2 agents)
$360
Self-hosted (any)
Linux runners
$0
$80 (controller + 2 agents)
$80 + admin
#
Self-hosted excludes operator time. For 10K monthly minutes, plan 3-5 hrs/mo of admin time, $450-750 in implicit cost.
Gradle cache strategy
The single most leveraged Android CI optimisation is aggressive Gradle caching. A well-configured cache strategy halves typical build time. Three layers matter.
First, the Gradle dependency cache. The standard actions/cache or vendor-equivalent cache action stores the .gradle/caches directory between runs, restored on subsequent jobs by branch and Gradle file content hash. A typical cache hit saves 60-90 seconds of dependency resolution per run. Cache miss falls back to full download, which is fine but slow.
Second, the Gradle build cache. This caches compiled classes, R.java generation outputs, and Kotlin compilation results. Properly configured (gradle.properties: org.gradle.caching=true) plus a remote build cache backend (local file, S3, Develocity), build-time savings on cache-hit branches can be 50-70 percent.
Third, the Gradle configuration cache (still in incubation in some Gradle versions but stable for most use). Caches the result of project configuration evaluation. Saves 10-30 seconds per run on warm cache, more on complex multi-module builds. Some plugins are not yet configuration-cache compatible; check before enabling globally.
All three combined typically take a 10-minute build down to 4-5 minutes on cache-hit runs, which translates directly into 50 percent fewer billable minutes on per-minute platforms. For a 10-developer team consuming 9,300 monthly minutes, optimising caches drops that to 5,000-6,000 minutes, dropping the GitHub Actions overage line item by $20-30 per month. Compound across team growth and the savings become substantial.
Emulator handling
Android instrumentation tests run inside an emulator, which has to boot and warm up before the tests start. On hosted Linux runners without KVM acceleration, emulator boot takes 60-120 seconds and the tests themselves run 2-3x slower than on hardware-accelerated emulators. The reactivecircus/android-emulator-runner Action is the de facto standard for hosted Actions; it manages emulator lifecycle, including snapshot caching to skip cold boots.
For teams running serious instrumentation test volumes, self-hosted runners with KVM acceleration are 2-3x faster than hosted equivalents. A self-hosted Linux VM with nested virtualisation enabled (typical on bare-metal or cloud instances that expose KVM) runs Android emulators near native speed. Combined with the cost saving of self-hosted runners, this is often the single biggest Android CI win.
Alternatively, push instrumentation tests to Firebase Test Lab. Real-device tests run on Google's hardware, billed per device-hour ($1 for physical, $5 for virtual on higher tiers). For teams that need real-device coverage anyway, Test Lab is often cheaper per test run than maintaining a self-hosted emulator fleet, and produces results that are more representative of production behaviour.
Recommendation by team shape
Solo or 2-5 person Android team: GitHub Actions Team or Free, with aggressive Gradle caching from the start. Cost stays under $100 monthly for typical workloads.
10-25 person Android-shipping team: GitHub Actions Team plus self-hosted Linux runners for instrumentation tests. The hybrid model captures the elasticity and ecosystem benefit of hosted while pulling the bulk of compute cost down through self-hosted.
Mobile-first team running both iOS and Android: Bitrise unifies the workflow library across both stacks at a per-minute premium that is more justifiable for the iOS half. Or Buildkite with self-hosted Linux for Android plus self-hosted Mac fleet for iOS, which is cheaper at scale but requires more platform engineering.
Above 50 Android developers or 50,000 monthly Linux minutes: self-hosted runners on the platform of your choice (Actions, GitLab, Buildkite). The per-minute math justifies the operator overhead at this scale.
Related deep dives
iOS CI cost
Java / JVM build cost
Bitrise pricing
cost optimization
self-hosted vs cloud
cost calculator
Frequently Asked Questions
#
click any question to expand
Why are Android CI builds slow?
>
Three reasons compound. Gradle daemon initialisation takes 30-60 seconds on a cold runner. Dependency resolution and build-config evaluation adds another 1-3 minutes for non-trivial projects. Android emulator boot for instrumentation tests adds 60-120 seconds per emulator instance. A simple Android pipeline running unit tests plus a single emulator integration test typically consumes 8-15 minutes per CI run, double or triple the equivalent backend pipeline.
What does an Android pipeline cost monthly?
>
A 10-developer Android team running 30 daily builds at 10-minute average consumes 6,300 monthly Linux build minutes. On GitHub Actions Team plan, that is around $33 in overage on top of $40 in seats, total $73 monthly. On CircleCI Performance, costs are inside the included credit allowance, total $150 monthly. On Bitrise, around 630 monthly workflow runs lands on the Pro 750-builds tier at a flat $655 (tier ladder confirmed by Bitrise, Aug 2026), which is why Android-only teams rarely pick it on cost. Self-hosted Linux runners absorb the load for around $50 monthly in infrastructure plus operator time.
Should I run instrumentation tests in CI?
>
Yes for confidence, but be deliberate about volume. Emulator-based instrumentation tests are slow (3-5 minutes per test class with cold-boot emulator) and resource-hungry (each emulator instance needs 4 GB RAM minimum, often 8 GB). Run a fast unit test suite on every PR and a slower instrumentation suite on merge to main or nightly. Tools like Firebase Test Lab or AWS Device Farm let you run real-device tests outside the CI pipeline, which is often a better fit for the slower work.
Does Gradle cache really matter?
>
Yes, more than any other single optimisation. A cold Gradle build downloads dependencies, evaluates buildSrc and applies plugins from scratch, easily adding 4-6 minutes. A warm Gradle build with proper dependency cache and configuration cache often completes the same work in 30-60 seconds. On Linux 2-core runners that is 60-80 percent of build time saved, immediately translating into billable-minute reduction on per-minute platforms.
Is Bitrise worth it for Android-only teams?
>
Less so than for iOS-only teams. Android does not have the macOS pricing penalty that drives iOS to Bitrise. On raw compute, Linux M on Bitrise ($0.0044 per minute) is actually slightly cheaper than GitHub Actions Linux ($0.006 per minute since January 2026), but Bitrise layers a flat plan fee ($99 Starter and up) on top. Bitrise's value for Android teams is mostly the workflow library (Gradle setup, signing, Play Store deploy steps), which can save pipeline-assembly time but does not justify the plan fee for an Android-only team that does not also need the mobile-specific automation.
Can I run Android emulators on GitHub Actions hosted runners?
>
Yes via the reactivecircus/android-emulator-runner action, which is the de facto standard. The action installs Android SDK components and boots an emulator inside the hosted Linux runner. Performance is acceptable but not great because hosted runners do not expose KVM nested virtualisation cleanly, so the emulator runs in software-rendered mode. A typical instrumentation test on a hosted GitHub Actions Linux runner takes 2-3x longer than on a self-hosted runner with KVM acceleration.
What about Firebase Test Lab integration?
>
Firebase Test Lab runs Android tests on real devices in Google's cloud. Pricing on the pay-as-you-go Blaze plan is $1 per device-hour for virtual devices and $5 per device-hour for physical devices (firebase.google.com/pricing, retrieved August 2026). For Android-shipping teams that need real-device confidence (especially for fragmentation testing across Android OEMs), Test Lab integrates cleanly with most CI/CD platforms via either the Firebase CLI or vendor-specific actions. Cost is meaningful but real-device test results are worth more than emulator results for production-bound code.
```

## java (/tmp/calc_java.html)

```text
Java CI/CD Cost 2026: Gradle, Maven, and JVM Build Economics | CICDCalculator.com
$
~/
cicdcalculator
[ menu ]
#
Independent CI/CD-cost research, read by platform and DevOps teams.
Sponsor this site →
Java and JVM CI/CD cost in 2026
./jvm-ci
--gradle --maven --where-mins-go
JVM CI builds are slower than they look. Gradle and Maven both have warmup overhead, dependency resolution that scales poorly with project size, and annotation-processing-heavy frameworks like Spring Boot or Quarkus that compound compilation time. The result is build minutes that consume more CI budget than equivalent pipelines in Go or Node. The good news: Gradle's build cache and configuration cache, properly configured, recover most of the slowness. The bad news: most teams do not configure them properly. This page works through where the time and money goes in a typical JVM CI pipeline in 2026 and how to claw it back.
Pricing references each vendor's public pricing page, re-verified August 2026 (TeamCity dollar figures excepted; the JetBrains store is JS-rendered and was not re-verifiable, so those stand at their last-verified values). Gradle build cache documentation at
docs.gradle.org
.
Where JVM build minutes go
A typical Spring Boot pipeline running compile, test and package consumes time in this approximate distribution on a Linux 2-core hosted runner without warm caches:
JVM startup + Gradle daemon init:
30-60 sec
Dependency resolution + download:
60-180 sec
Compile main + tests:
60-150 sec
Run unit test suite:
120-300 sec
Package JAR:
20-40 sec
Cold pipeline total:
~6-12 min
With proper cache:
Skip dependency download:
save 60-160 sec
Skip incremental compile:
save 30-90 sec
Warm pipeline total:
~3-5 min
The 50 percent reduction from warm caches is the single biggest CI optimisation a JVM team can make. It costs nothing to enable (the cache action and Gradle build-cache config are both free) and pays back immediately on per-minute platforms.
Per-platform monthly cost for a JVM team
Numbers below assume a 15-developer Java team consuming 8,400 monthly Linux build minutes (50 daily builds at 8-minute warm-cache average over 21 working days).
Platform
Plan
Seats
Compute
Total / mo
GitHub Actions
Team
$60
$32 (5.4K over 3K @ $0.006)
$92
GitLab CI
Premium
$435
$0 (in 10K free)
$435
TeamCity
Cloud (15 users)
$675
$0 (in credits)
$675
TeamCity
Server (5 agents)
$0 (license $300 / mo amort.)
$140 (infra)
$440 + admin
CircleCI
Performance
$225
$0 (in 375K credits)
$225
Jenkins
Self-hosted
$0
$120 (controller + 3 agents)
$120 + admin
The TeamCity premium, justified
TeamCity Cloud at $675 monthly looks expensive next to GitHub Actions at $92. The premium is real and only makes sense for teams that genuinely benefit from the TeamCity-IntelliJ integration. For a 15-developer Java team where every engineer uses IntelliJ Ultimate (already paying $599 per user per year, $7,485 annually for 15 seats), TeamCity is roughly $8,100 annually adding 8 percent to the total JetBrains spend.
The benefit is the Gradle and Maven first-class support: parallel build distribution across agents, detailed dependency analysis, build configuration that opens directly in IntelliJ for debugging, and Kotlin DSL for pipeline definitions that engineers already know from their build.gradle.kts files. For a team that values uniform tooling across IDE and CI, the premium is justified.
For a team that does not specifically value JetBrains integration, GitHub Actions is meaningfully cheaper at any team size and the JVM-specific advantage TeamCity offers is small. Most Java shops adopting CI/CD greenfield in 2026 land on GitHub Actions or GitLab CI; TeamCity adoption is concentrated in JetBrains-loyal organisations.
Build cache strategies in detail
For Gradle: enable the build cache (org.gradle.caching=true in gradle.properties) and configure a remote build cache backend. Local file backend works for small teams; for distributed teams, an S3-backed cache (via the gradle-build-cache-s3 plugin) lets every CI run benefit from work done by other developers' machines. Configuration cache (org.gradle.configuration-cache=true) saves 10-30 seconds per build but requires plugin compatibility checks.
For Maven: cache the entire ~/.m2/repository directory between CI runs, restored on Maven file content hash. Enable the parallel build flag (-T 1C) to use all available CPU cores. Maven does not have a build cache equivalent to Gradle's, so the win is mostly in dependency download avoidance and parallel execution.
For multi-module monorepos: invest in change-detection logic that skips unchanged modules. The reactivecircus/android-cache-fix-gradle-plugin (also useful for non-Android projects) helps with this. Or use Develocity's predictive test selection if budget allows. The best Java CI shops have made this work routine; the result is a typical PR build of 2-4 minutes regardless of monorepo size.
JVM-specific considerations
JVM warmup is real but mostly invisible on per-build pricing. The JVM compiles bytecode to optimised native code in the first few seconds of execution; workloads benefit cumulatively as the same code paths run more times. CI builds typically run each JVM process for 30-300 seconds, which is too short for full JIT warmup. This is not a cost lever in itself but it means JVM CI builds are slightly slower per workload than the same logic in a precompiled language like Go or Rust.
For workloads that genuinely benefit from native compilation (Spring Boot AOT mode, Quarkus native, GraalVM native-image), the build itself becomes much more expensive. Native-image compilation can take 5-15 minutes per artifact. Run native builds on dedicated CI workflows, not on every PR; merge-only or nightly is typical. The runtime startup benefit pays back in production but not in CI minutes consumed.
Larger heap settings on CI runners (-Xmx4g typical for medium projects) help with parallel test execution but require runners with adequate RAM. Linux 2-core runners on most platforms have 7 GB RAM available, enough for typical JVM CI workloads. Larger projects (50+ modules, 10+ GB heap usage during tests) may need 4-core or 8-core runners with more RAM, which doubles or quadruples the per-minute cost.
Related deep dives
TeamCity pricing
Android CI cost (also Gradle)
cost optimization
Jenkins vs Bamboo
compare all platforms
cost calculator
Frequently Asked Questions
#
click any question to expand
Why are JVM CI builds slower than Node or Go?
>
Three reasons. JVM warmup adds 5-15 seconds before any compilation starts, which compounds across multi-step pipelines. Gradle daemon initialisation adds another 30-60 seconds on cold runs. Annotation-processing-heavy frameworks (Spring Boot, Quarkus, Lombok) trigger expensive recompilation cycles, especially with KAPT. A typical Spring Boot pipeline that compiles, runs unit tests and packages a JAR consumes 6-12 minutes of CI time, roughly double the equivalent Go or Node pipeline.
What does a Java pipeline cost monthly?
>
A 15-developer Java team running 50 daily builds at 8-minute average consumes 8,400 monthly Linux build minutes. On GitHub Actions Team plan, that is around $32 in overage (5,400 minutes over the included 3,000 at $0.006 since the January 2026 rate cut) on top of $60 in seats, total $92 monthly. On TeamCity Cloud at $45 per user including credits, total $675 monthly (the price is the JetBrains tooling and integration value, not raw CI/CD cost). On self-hosted Jenkins or GitLab CE, around $100 monthly in infrastructure plus operator time.
Is Gradle or Maven cheaper to run in CI?
>
Gradle is faster per build with proper cache configuration; Maven is more predictable. A warm Gradle build with build cache enabled can be 30-50 percent faster than the equivalent Maven run because Gradle skips work that has not changed. Maven is more deterministic and easier to cache because the dependency model is simpler. For new Java projects starting in 2026, Gradle is usually the right choice for CI speed; for legacy Maven shops, the migration cost typically outweighs the per-build saving.
Why is TeamCity popular in JVM shops?
>
JetBrains is the dominant Java tooling vendor (IntelliJ IDEA), and the IntelliJ-TeamCity integration is genuinely tight. Build configurations open directly in IntelliJ for debugging. Pipeline-level annotations show up in the IDE. The first-class Gradle and Maven support includes parallel build distribution and detailed dependency analysis that other CI/CD platforms lack. For Java teams already paying for IntelliJ Ultimate, the marginal cost of TeamCity is small and the integration benefit is real.
Should I cache the JVM itself?
>
If you are using a JVM with native compilation (GraalVM native-image, AOT-compiled Java), absolutely. Native-image compilation can take 5-15 minutes and the result is large but caches well. For standard JIT-compiled Java, the JVM itself is small enough that re-extracting it on each run is cheap; the wins come from caching the dependency graph (Gradle / Maven cache) rather than the JVM itself. Distribution-class JVMs (Temurin, Corretto, GraalVM CE) all install fast on hosted runners.
What about multi-module Gradle projects?
>
Multi-module Gradle projects are where the Gradle build cache delivers the largest wins. A 50-module project building only the changed modules can complete in 2-3 minutes versus 15-20 minutes for a full build. Configure the build cache to skip unchanged modules; combine with conditional CI jobs that only test the modules a PR touches. Most large Java shops invest heavily in this kind of incremental-CI tooling because it is the single biggest CI cost lever for monorepo-style codebases.
Is Develocity worth the cost?
>
Develocity (formerly Gradle Enterprise) is Gradle's commercial build-acceleration product. It provides a remote build cache, build scans for performance analysis, predictive test selection and developer-facing performance reporting. Pricing is sales-led, typically starting around $80-150 per user per month for medium teams. The value depends on team size and codebase complexity. For 50+ developer Java shops with large multi-module Gradle builds, the per-build time savings often pay for the licence within the first few months. For smaller teams or simpler builds, the open-source Gradle build cache is sufficient.
```

## migration (/tmp/calc_migration.html)

```text
CI/CD Migration Cost Calculator 2026: Switch Platforms | CICDCalculator.com
$
~/
cicdcalculator
[ menu ]
#
Independent CI/CD-cost research, read by platform and DevOps teams.
Sponsor this site →
CI/CD migration estimator
./migrate
--from $A --to $B
Switching CI/CD platforms is rarely a weekend project. This calculator estimates engineer-hours, calendar time, cost, and risk for the most common migration paths, based on observed averages from team retrospectives.
~/cicdcalculator/migrate --estimate
live
from platform
GitHub Actions
GitLab CI
CircleCI
Jenkins
Buildkite
Bitbucket Pipelines
to platform
GitHub Actions
GitLab CI
CircleCI
Jenkins
Buildkite
Bitbucket Pipelines
number of pipelines
devops engineers on the project
hourly rate ($ loaded)
$
result.json
{
"from":
"
Jenkins
"
,
"to":
"
GitHub Actions
"
,
"per_pipeline_hours":
6
,
"pipeline_hours":
72
,
"setup_hours":
60
,
"total_hours":
132
,
"timeline_weeks":
3
,
"risk":
"
high
"
,
"cost_usd":
13,200
}
Estimate uses observed averages from public engineering blog posts and team retrospectives. Real numbers vary by pipeline complexity, monorepo vs polyrepo structure, and whether you rebuild pipelines from scratch versus translating yaml.
Common migration paths
From
To
Typical cost
Calendar weeks
Notes
Jenkins
GitHub Actions
$8-25K
8-12
Most common migration. Plan for plugin-by-plugin replacement, secrets rotation, and a parallel-run period.
Jenkins
GitLab CI
$10-30K
10-14
Often paired with a move to GitLab source control. Heavy if your Jenkins has lots of shared libraries.
GitLab CI
GitHub Actions
$5-15K
4-8
YAML-to-YAML, similar concepts. Watch for GitLab-specific features (DAG, dynamic envs).
CircleCI
GitHub Actions
$4-12K
3-6
Most teams' first CI move. orb-to-action mapping needs care; resource_class becomes runner labels.
Bitbucket
GitHub Actions
$6-15K
5-9
Often coupled with a code-host move. Budget time for pipeline rewrites and team training.
Anywhere
Buildkite
$10-30K
6-12
Adds the cost of standing up agent infrastructure. Pays back at scale.
How to plan the migration
01.
Inventory.
Count your pipelines, list custom plugins or actions, document secrets, identify the half-dozen jobs that absolutely cannot break.
02.
Pick a pilot.
Choose a low-risk pipeline (a small service, a docs site) and migrate it end-to-end. Document everything you learn.
03.
Migrate in shadow mode.
Run both systems in parallel for 2-4 weeks. The old platform remains the gate; the new one is observed.
04.
Rotate every secret.
Don't copy-paste. Every secret gets a fresh value in the new platform's secret store.
05.
Cut over by team.
One team at a time. Each team flips their merge gate to the new platform once they're confident.
06.
Decommission carefully.
Keep the old platform running read-only for at least 30 days for archive access and rollback. Then turn it off.
Related
platform comparison
optimization tips
jenkins TCO
cost calculator
Frequently Asked Questions
#
click any question to expand
How long does a CI/CD migration take?
>
For a mid-size team migrating 10-20 pipelines from Jenkins to a hosted platform, plan 6-12 weeks of part-time work for two engineers. Cloud-to-cloud migrations (GitLab to GitHub Actions, for example) usually take 3-6 weeks because the YAML structures are similar. Migrations away from Jenkins always run longer due to plugin sprawl and the variety of custom Groovy logic that accumulates.
Should I rewrite pipelines or auto-translate them?
>
Auto-translation tools (GitLab's importer, GitHub Actions Importer, atomicgo's pipeline-converter) handle the common case but produce messy yaml that's hard to maintain. For long-lived pipelines we recommend treating migration as a chance to simplify: rewrite from scratch using current best practices, dropping the accumulated workarounds. The extra time pays back in maintainability.
What's the riskiest part of CI/CD migration?
>
Secrets and credentials. Every CI/CD platform has its own secrets store, and accidentally committing a secret while moving between systems is a real risk. Plan a separate secrets-rotation step where every secret is rotated and re-issued in the new system, never copy-pasted. Also watch for environment-specific URLs and config drift between staging and production pipelines.
Can I run two CI/CD systems in parallel during migration?
>
Yes and you should. Most teams run the new platform in shadow mode for 2-4 weeks: builds happen on both, but the old system remains the source of truth for merge gates. Once the new system has matched the old for a couple of weeks without surprises, flip the gate. This staged cutover prevents PR queues from grinding to a halt during the move.
When is migration not worth it?
>
When your current platform works, your team is happy, and the only motivation is cost. The savings need to justify 3-12 weeks of engineering time plus production risk. A useful rule: only migrate if the projected annual savings exceed 2x the migration cost. Otherwise spend the same engineering time on optimisation inside your current platform.
Should I migrate to a self-hosted Jenkins from a cloud CI?
>
Rarely. Most teams migrate the other way (off Jenkins, onto a hosted platform). The case for moving to Jenkins is compliance (data sovereignty, audited build environments) or extreme scale. If neither applies, the operator burden of Jenkins outweighs any per-minute savings versus a hosted platform.
```
