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
