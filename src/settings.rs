
// 19 workflows - dated 240118
const WORKFLOWS: &'static [&'static str] = 
    &["https://github.com/epi2me-labs/wf-16s",
    "https://github.com/epi2me-labs/wf-aav-qc",
    "https://github.com/epi2me-labs/wf-alignment",
    "https://github.com/epi2me-labs/wf-amplicon",
    "https://github.com/epi2me-labs/wf-artic", 
    "https://github.com/epi2me-labs/wf-bacterial-genomes",
    "https://github.com/epi2me-labs/wf-basecalling",
    "https://github.com/epi2me-labs/wf-cas9",
    "https://github.com/epi2me-labs/wf-clone-validation",
    "https://github.com/epi2me-labs/wf-flu",
    "https://github.com/epi2me-labs/wf-human-variation",
    "https://github.com/epi2me-labs/wf-metagenomics",
    "https://github.com/epi2me-labs/wf-mpx",
    "https://github.com/epi2me-labs/wf-pore-c",
    "https://github.com/epi2me-labs/wf-single-cell",
    "https://github.com/epi2me-labs/wf-somatic-variation",
    "https://github.com/epi2me-labs/wf-tb-amr", 
    "https://github.com/epi2me-labs/wf-template",
    "https://github.com/epi2me-labs/wf-transcriptomes",
    ];

    pub fn available_workflows() -> Vec<String> {
        let wf_projects: Vec<String> = WORKFLOWS.iter().map(|v| v.to_string()).collect();
        return wf_projects;
    }

    pub fn list_available_workflows()  -> Vec<String> {
        let workflow_urls = available_workflows();
        let clipped: Vec<String> = workflow_urls.iter().map(|v| { 
            v.replace("https://github.com/", "") }).collect();
        return clipped;
    }