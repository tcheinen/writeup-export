use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::format;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "i", default_value = "in")]
    input_folder: String,
    #[structopt(short = "o", default_value = "out")]
    output_folder: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    for folder in std::fs::read_dir(&opt.input_folder)?
        .flatten()
        .filter(|x| x.file_type().unwrap().is_dir())
        .filter(|x| !x.file_name().to_string_lossy().contains(".git"))
    {
        let meta_path = {
            let mut folder_path = folder.path();
            folder_path.push("meta.toml");
            folder_path
        };

        let meta: CTFMeta = toml::from_str(&std::fs::read_to_string(meta_path)?)?;
        let challenges = meta
            .challenges
            .iter()
            .map(|(a, b)| ((b, a.clone()), a.clone() + ".md"))
            .map(|(a, b)| {
                let mut path = folder.path();
                path.push(b);
                (a, path)
            })
            .flat_map(|(a, b)| Some((a, std::fs::read_to_string(b).ok()?)))
            .collect::<Vec<_>>();

        let front_matter = format!(
            "+++\ntitle=\"{}\"\ndate = {}\n\n[taxonomies]\ntags = [\"ctf-writeups\"]\n+++\n",
            &meta.name, &meta.date
        );
        let description = meta.description.map(|desc| desc + "\n<!-- more -->\n");

        let section_page = front_matter
            + &description.unwrap_or(String::new())
            + &challenges
                .iter()
                .map(|((cmeta, _), b)| format!("# {}\n{}", cmeta.name, b))
                .collect::<Vec<_>>()
                .join("\n")
                .replace("\n#", "\n##");

        let challenge_pages = challenges.into_iter().map(|((cmeta, name), content)| {
            (
                (cmeta, name),
                format!(
                    "+++\ntitle=\"{}\"\ndate = {}\n\n[taxonomies]\ntags = [{}]\n+++\n\n\n{}",
                    &cmeta.name,
                    &meta.date,
                    cmeta
                        .tags
                        .as_ref()
                        .unwrap_or(&vec![])
                        .into_iter()
                        .map(|x| format!("{:?}", x))
                        .collect::<Vec<_>>()
                        .join(","),
                    content
                ),
            )
        });
        let section_path = {
            let mut section_path = PathBuf::new();
            section_path.push(&opt.output_folder);
            section_path.push(folder.file_name().to_string_lossy().to_string());
            section_path
        };
        std::fs::create_dir(&section_path);
        let index_path = {
            let mut index_path = section_path.clone();
            index_path.push("index.md");
            index_path
        };
        std::fs::write(index_path, section_page)?;
        for ((cmeta, name), content) in challenge_pages {
            let chal_path = {
                let mut chal_path = section_path.clone();
                chal_path.push(format!("{}.md", name));
                chal_path
            };
            std::fs::write(chal_path, content)?;
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CTFMeta {
    name: String,
    date: String,
    description: Option<String>,
    challenges: HashMap<String, ChallengeMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeMeta {
    name: String,
    tags: Option<Vec<String>>,
}

mod test {
    use super::*;

    #[test]
    fn parse_meta() {
        let meta = "
name = \"Test!\"

[challenges]
    [challenges.example]
        name = \"Challenge 1\"
";

        let meta: CTFMeta = toml::from_str(meta).unwrap();
        println!("{:?}", meta);
        assert!(false);
    }
}
