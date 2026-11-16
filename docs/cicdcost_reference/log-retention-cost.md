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
