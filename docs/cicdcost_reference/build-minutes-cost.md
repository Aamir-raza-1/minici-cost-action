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
