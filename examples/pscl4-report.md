# pSCL4 deleted-arm BGC gene-level competition analysis

## Scope

- Reference: `JIC2020 / NZ_CP045850.1 (pSCL4)`.
- Candidate regions: pSCL4 regions previously mapped from KAIST `NZ_CP027859.1` antiSMASH output and classified as `lost_in_industrial` or `partially_lost`.
- Comparison target: clavulanic-acid-related pathways curated in `BactGenome_analysis_JIC2020/10_clav_bgc_variants`.
- Interpretation level: precursor/cofactor/regulatory competition is **inference from gene content**, not direct flux proof.

## Clav Pathway Competition Axes

| Axis | Type | Clav evidence | Representative genes |
| --- | --- | --- | --- |
| arginine_ornithine_agmatine_nitrogen | direct_precursor | argJ/oat1 + speB + multiple aminotransferases feed the C5-N ornithine/arginine/agmatine branch | GE265_RS20195(argJ), GE265_RS20205(speB), GE265_RS30820(oat1), GE265_RS30825(speB), GE265_RS30840/30850(aminotransferases) |
| tpp_pyruvate_g3p_entry | direct_precursor | ceaS1/ceaS2 are TPP-dependent entry enzymes consuming pyruvate/glyceraldehyde-3-phosphate equivalents | GE265_RS20215(ceaS2), GE265_RS30835(ceaS1) |
| plp_aminotransfer_redirection | cofactor_plus_precursor | lat and several aminotransferases impose PLP-dependent nitrogen transfer demand | GE265_RS20235(lat), GE265_RS13990/14035, GE265_RS30815/30840/30850 |
| redox_oxygenation | cofactor | clav pathway contains P450s, SDRs, flavin reductase, ferredoxin and dioxygenase-type steps | GE265_RS20125, GE265_RS20175, GE265_RS20180, GE265_RS14020, GE265_RS14010, GE265_RS20225 |
| atp_adenylation_export | energetic | ATP-grasp, transport, and peptide assembly impose ATP demand on top of biosynthesis | GE265_RS20140, GE265_RS20150/20190, GE265_RS20230, multiple plasmid-borne paralog enzymes |
| pathway_regulation | regulatory | ccaR/claR plus plasmid-borne res1/res2/snk suggest production is sensitive to global regulator competition | GE265_RS20250(ccaR), GE265_RS20185(claR), GE265_RS30795/30800/30805, GE265_RS14025 |

## Clav Cluster Context

| Cluster | Gene count | Representative functions | Competition relevance |
| --- | ---: | --- | --- |
| ca_biosynthetic_cluster_jic2020 | 29 | AfsR/SARP family transcriptional regulator; LysR family transcriptional regulator; SDR family oxidoreductase; argJ; asparagine synthase-related protein; cs1; cytochrome P450; isopenicillin N synthase family dioxygenase; lat; response regulator transcription factor; sensor histidine kinase; speB | Chromosomal CA core cluster with ArgJ/SpeB/CeaS2/Lat plus CcaR/ClaR-linked control. |
| plasmid_borne_paralogous_clavaminic_acid_biosynthe_jic2020 | 13 | BTAD domain-containing putative transcriptional regulator; GAF domain-containing sensor histidine kinase; PLP-dependent aminotransferase family protein; aminotransferase family protein; asparagine synthase-related protein; bifunctional ornithine acetyltransferase/N-acetylglutamate synthase; pyridoxal phosphate-dependent aminotransferase; response regulator; speB | pSCL4 plasmid-borne paralog block with oat1/speB/bls1/ceaS1 plus res1/res2/snk. |
| separate_clavam_cluster_jic2020 | 17 | BTAD domain-containing putative transcriptional regulator; LLM class flavin-dependent oxidoreductase; aminotransferase family protein; aminotransferase-like domain-containing protein; cs1 | Separate clavam branch enriched in aminotransferase, oxidoreductase and regulator functions. |

## Candidate pSCL4 Deleted / Partially Lost BGCs

| Region | Products | pSCL4 coords | Industrial status | Competition level | Top axes | Interpretation |
| --- | --- | --- | --- | --- | --- | --- |
| NZ_CP027859.1.region017 | NRPS-like,terpene | 67731-125794 | lost_in_industrial | high_priority_competitor_candidate | amino_acid_or_nitrogen_precursors,regulatory_crosstalk_possible,redox_and_oxygenation,atp_and_activation_burden | Left-arm NRPS/NRPS-like + terpene islands with regulatory modules; likely moderate competition and signaling burden. |
| NZ_CP027859.1.region016 | terpene,NRPS | 138446-200209 | lost_in_industrial | high_priority_competitor_candidate | redox_and_oxygenation,acetyl_coa_malonyl_coa_isoprenoid_pool,regulatory_crosstalk_possible,atp_and_activation_burden | Left-arm NRPS/NRPS-like + terpene islands with regulatory modules; likely moderate competition and signaling burden. |
| NZ_CP027859.1.region015 | NRPS-like,T1PKS,phosphoglycolipid,NRPS,ectoine | 236931-402468 | partially_lost | high_priority_competitor_candidate | amino_acid_or_nitrogen_precursors,atp_and_activation_burden,redox_and_oxygenation,sam_methylation_or_radical_chemistry | Mixed NRPS/T1PKS/ectoine region spanning the left-arm deletion boundary; strongest combined amino-acid, malonyl-CoA, ATP and redox burden candidate. |
| NZ_CP027859.1.region006 | terpene,lanthipeptide-class-iv | 1123806-1151421 | lost_in_industrial | moderate_priority_competitor_candidate | atp_and_activation_burden,regulatory_crosstalk_possible,acetyl_coa_malonyl_coa_isoprenoid_pool,sam_methylation_or_radical_chemistry | Mostly terpene-focused island; likely indirect burden through isoprenoid/redox demand rather than direct clav precursor competition. |
| NZ_CP027859.1.region005 | terpene-precursor,indole,NRPS-like,terpene | 1195266-1269220 | lost_in_industrial | high_priority_competitor_candidate | redox_and_oxygenation,acetyl_coa_malonyl_coa_isoprenoid_pool,regulatory_crosstalk_possible,amino_acid_or_nitrogen_precursors | Indole + NRPS-like + terpene region suggesting aromatic amino-acid, prenyl donor and redox demand. |
| NZ_CP027859.1.region004 | terpene | 1302208-1329332 | lost_in_industrial | indirect_or_lower_priority_competitor_candidate | acetyl_coa_malonyl_coa_isoprenoid_pool,atp_and_activation_burden,redox_and_oxygenation,sam_methylation_or_radical_chemistry | Mostly terpene-focused island; likely indirect burden through isoprenoid/redox demand rather than direct clav precursor competition. |
| NZ_CP027859.1.region003 | lassopeptide | 1381389-1403856 | lost_in_industrial | moderate_priority_competitor_candidate | atp_and_activation_burden,amino_acid_or_nitrogen_precursors,regulatory_crosstalk_possible | Lassopeptide region likely consumes peptide maturation capacity and ATP, but direct clav precursor overlap is limited. |
| NZ_CP027859.1.region002 | NRPS,NRPS-like,terpene | 1459946-1539100 | lost_in_industrial | high_priority_competitor_candidate | amino_acid_or_nitrogen_precursors,atp_and_activation_burden,sam_methylation_or_radical_chemistry,plp_dependent_nitrogen_transfer | Multi-NRPS/siderophore-like region with panD, ornithine carbamoyltransferase, sbnA-like genes and P450; strongest direct nitrogen/amino-acid competition candidate. |
| NZ_CP027859.1.region001 | terpene | 1693518-1718466 | lost_in_industrial | indirect_or_lower_priority_competitor_candidate | redox_and_oxygenation,acetyl_coa_malonyl_coa_isoprenoid_pool,sam_methylation_or_radical_chemistry | Mostly terpene-focused island; likely indirect burden through isoprenoid/redox demand rather than direct clav precursor competition. |

## Priority Interpretation

1. `region015` is the strongest candidate for releasing competitive burden because it combines `NRPS-like + T1PKS + NRPS + ectoine` functions and straddles the deletion boundary, directly implicating amino-acid, ATP, malonyl-CoA, redox, and osmoprotectant nitrogen allocation.
2. `region002` is the strongest fully lost nitrogen-demanding competitor because it contains multiple NRPS enzymes plus `panD`, `ornithine carbamoyltransferase`, `sbnA`-like functions and redox tailoring enzymes, giving the clearest overlap with clav nitrogen and ATP demand.
3. `region005` is a strong secondary candidate because it combines `NRPS-like`, `indole`, `terpene`, aminotransferase, and two-component regulatory modules; this suggests competition for aromatic amino acids, prenyl donors, ATP and redox capacity.
4. `region016` and `region017` are moderate candidates: both are left-arm losses with `NRPS/NRPS-like + terpene` logic and regulator modules, but their direct overlap with the clav-specific ornithine/agmatine branch is weaker than `region002` or `region015`.
5. `region001`, `region004`, and much of `region006` are mainly terpene-oriented. Their loss likely reduces indirect isoprenoid/redox burden more than it releases the most clav-specific precursor pools.
6. `region003` lassopeptide loss is real, but likely contributes less to clav overproduction than the larger multi-enzyme NRPS/PKS islands.

## Region Notes

### NZ_CP027859.1.region017

- Products: `NRPS-like,terpene`
- pSCL4 coordinates: `67731-125794`
- Industrial state: `lost_in_industrial`
- Local industrial context: `wild_retained_industrial_lost_or_fragmented:49`
- Highlight genes: CRV15_RS35180: response regulator transcription factor | CRV15_RS37630: sensor histidine kinase
- Interpretation: Left-arm NRPS/NRPS-like + terpene islands with regulatory modules; likely moderate competition and signaling burden.

### NZ_CP027859.1.region016

- Products: `terpene,NRPS`
- pSCL4 coordinates: `138446-200209`
- Industrial state: `lost_in_industrial`
- Local industrial context: `wild_retained_industrial_lost_or_fragmented:42`
- Highlight genes: CRV15_RS34920: terpene synthase family protein | CRV15_RS34930: cytochrome P450 | CRV15_RS35020: non-ribosomal peptide synthetase | CRV15_RS35030: terpene synthase family protein | CRV15_RS35070: ATP-binding protein | CRV15_RS35075: response regulator transcription factor
- Interpretation: Left-arm NRPS/NRPS-like + terpene islands with regulatory modules; likely moderate competition and signaling burden.

### NZ_CP027859.1.region015

- Products: `NRPS-like,T1PKS,phosphoglycolipid,NRPS,ectoine`
- pSCL4 coordinates: `236931-402468`
- Industrial state: `partially_lost`
- Local industrial context: `wild_retained_industrial_lost_or_fragmented:97,retained_in_both:12`
- Highlight genes: CRV15_RS34285: type I polyketide synthase | CRV15_RS34320(asnB): asparagine synthase (glutamine-hydrolyzing) | CRV15_RS34330: asparagine synthetase B family protein | CRV15_RS34410: ectoine synthase | CRV15_RS34440: non-ribosomal peptide synthetase | CRV15_RS34445: non-ribosomal peptide synthetase | CRV15_RS34465: cytochrome P450 | CRV15_RS34610: non-ribosomal peptide synthetase | CRV15_RS34615: non-ribosomal peptide synthetase/MFS transporter | CRV15_RS34655: type I polyketide synthase | CRV15_RS34670: cytochrome P450
- Interpretation: Mixed NRPS/T1PKS/ectoine region spanning the left-arm deletion boundary; strongest combined amino-acid, malonyl-CoA, ATP and redox burden candidate.

### NZ_CP027859.1.region006

- Products: `terpene,lanthipeptide-class-iv`
- pSCL4 coordinates: `1123806-1151421`
- Industrial state: `lost_in_industrial`
- Local industrial context: `wild_retained_industrial_lost_or_fragmented:17`
- Highlight genes: CRV15_RS37130: histidine kinase | CRV15_RS31155: terpene synthase family protein | CRV15_RS31175(lanL): class IV lanthionine synthetase LanL
- Interpretation: Mostly terpene-focused island; likely indirect burden through isoprenoid/redox demand rather than direct clav precursor competition.

### NZ_CP027859.1.region005

- Products: `terpene-precursor,indole,NRPS-like,terpene`
- pSCL4 coordinates: `1195266-1269220`
- Industrial state: `lost_in_industrial`
- Local industrial context: `wild_retained_industrial_lost_or_fragmented:53`
- Highlight genes: CRV15_RS30785: cytochrome P450 | CRV15_RS37550: terpene synthase family protein | CRV15_RS30870: cytochrome P450 | CRV15_RS30875: terpene synthase family protein | CRV15_RS30885: cytochrome P450 | CRV15_RS30905: response regulator | CRV15_RS30910: sensor histidine kinase
- Interpretation: Indole + NRPS-like + terpene region suggesting aromatic amino-acid, prenyl donor and redox demand.

### NZ_CP027859.1.region004

- Products: `terpene`
- pSCL4 coordinates: `1302208-1329332`
- Industrial state: `lost_in_industrial`
- Local industrial context: `wild_retained_industrial_lost_or_fragmented:21`
- Highlight genes: CRV15_RS30455: cytochrome P450 | CRV15_RS30465: labda-7,13(16),14-triene synthase
- Interpretation: Mostly terpene-focused island; likely indirect burden through isoprenoid/redox demand rather than direct clav precursor competition.

### NZ_CP027859.1.region003

- Products: `lassopeptide`
- pSCL4 coordinates: `1381389-1403856`
- Industrial state: `lost_in_industrial`
- Local industrial context: `wild_retained_industrial_lost_or_fragmented:23`
- Highlight genes: CRV15_RS30050: lasso peptide biosynthesis B2 protein | CRV15_RS30060: asparagine synthase-related protein
- Interpretation: Lassopeptide region likely consumes peptide maturation capacity and ATP, but direct clav precursor overlap is limited.

### NZ_CP027859.1.region002

- Products: `NRPS,NRPS-like,terpene`
- pSCL4 coordinates: `1459946-1539100`
- Industrial state: `lost_in_industrial`
- Local industrial context: `wild_retained_industrial_lost_or_fragmented:60`
- Highlight genes: CRV15_RS29510(panD): aspartate 1-decarboxylase | CRV15_RS29515: ornithine carbamoyltransferase | CRV15_RS29525: non-ribosomal peptide synthetase | CRV15_RS29530: non-ribosomal peptide synthetase | CRV15_RS29600: non-ribosomal peptide synthetase | CRV15_RS29610: non-ribosomal peptide synthetase | CRV15_RS29695: cytochrome P450 | CRV15_RS29700: (-)-delta-cadinene synthase
- Interpretation: Multi-NRPS/siderophore-like region with panD, ornithine carbamoyltransferase, sbnA-like genes and P450; strongest direct nitrogen/amino-acid competition candidate.

### NZ_CP027859.1.region001

- Products: `terpene`
- pSCL4 coordinates: `1693518-1718466`
- Industrial state: `lost_in_industrial`
- Local industrial context: `wild_retained_industrial_lost_or_fragmented:16`
- Highlight genes: CRV15_RS28460: cytochrome P450 | CRV15_RS28465: terpene synthase | CRV15_RS28470: cytochrome P450 | CRV15_RS28490: cytochrome P450 | CRV15_RS28510: cytochrome P450 family protein
- Interpretation: Mostly terpene-focused island; likely indirect burden through isoprenoid/redox demand rather than direct clav precursor competition.

