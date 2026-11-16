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
