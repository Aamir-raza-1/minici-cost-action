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
