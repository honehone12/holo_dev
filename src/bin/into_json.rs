use std::{fs, time};
use chrono::{FixedOffset, NaiveDate};
use scraper::{Html, Selector};
use anyhow::bail;
use tracing::{error, warn, info, debug};
use holo_dev::Song;

const SOURCE: &'static str = "html/original_music_list_utf8.html"; 
const DEST: &'static str = "intermediate/original_music_list.json";

fn scrape() -> anyhow::Result<Vec<Song>> {
    let document = fs::read_to_string(SOURCE)?;
    
    let music_div = match Selector::parse("div.wiki-section-3") {
        Ok(s) => s,
        Err(e) => bail!("{e}")
    };
    let title_div = match Selector::parse("div.title-3") {
        Ok(s) => s,
        Err(e) => bail!("{e}")
    };
    let body_div = match Selector::parse("div.wiki-section-body-3") {
        Ok(s) => s,
        Err(e) => bail!("{e}")
    };
    let link_a = match Selector::parse("a.outlink") {
        Ok(s) => s,
        Err(e) => bail!("{e}")
    };

    let document = Html::parse_document(&document);
    
    let Some(jst) = FixedOffset::east_opt(60 * 60 * 9) else {
        bail!("failed to create jst offset east 9 hour");
    };
    let mut songs = vec![];
    for div in document.select(&music_div) {
        
        let Some(title_elem) = div.select(&title_div).next() else {
            warn!("skipping element without title");
            continue;
        };
        let title_texts = {
            let mut v = vec![];
            for text in title_elem.text() {
                let t = text.trim();
                if !t.is_empty() {
                    v.push(t);
                }
            }
            v
        };
        if title_texts.len() != 1 {
            warn!("skipping elemt that title text is not 1");
            debug!("{title_texts:?}");
            continue;
        }
        let title = title_texts[0].to_string();
        
        let Some(body_elem) = div.select(&body_div).next() else {
            warn!("skipping element without body");
            continue;
        };
        let mut links = vec![];
        for a in body_elem.select(&link_a) {
            if let Some(l) = a.attr("href") {
                links.push(l.to_string());
            }
        }

        let mut properties = {
            let mut v = vec![];
            for text in body_elem.text() {
                let t = text.trim();
                if !t.is_empty() {
                    v.push(t.to_string());
                }
            }
            v
        };
        if properties.is_empty() {
            warn!("skipping element with empty body");
            continue;
        }

        let Some(idx) = properties.iter()
            .position(|p| p.starts_with("メンバー：")) 
        else {
            warn!("skipping elemnt that body does not have property 'メンバー：'");
            debug!("{properties:?}");
            continue;
        };
        let m = properties.remove(idx);
        let member = if m.contains(":") {
            m.split(":")
                .collect::<Vec<&str>>()[1]
                .trim()
                .to_string()
        } else if m.contains("：") {
            m.split("：")
                .collect::<Vec<&str>>()[1]
                .trim()
                .to_string()
        } else {
            warn!("skipping 'member' not containing ':' nor '：'");
            debug!("{m}");
            continue;
        };

        let published = match properties.iter()
            .position(|p| p.starts_with("音源公開日：")
        ) {
            Some(idx) => {
                let p = properties.remove(idx);
                let mut s = if p.contains(":") {
                    p.split(":")
                        .collect::<Vec<&str>>()[1]
                        .trim()
                } else if p.contains("：") {
                    p.split("：")
                        .collect::<Vec<&str>>()[1]
                        .trim()
                } else {
                    warn!("skipping 'published' not containing ':' nor '：'");
                    debug!("{p}");
                    continue;        
                };

                if s.len() > 10 {
                    let (d, ex) = s.split_at(10);
                    s = d;
                    warn!("removing from published date: {ex}");
                }

                let utc = match NaiveDate::parse_from_str(&s, "%Y/%m/%d") {
                    Ok(d) => {
                        let Some(ndt) = d.and_hms_opt(12, 0, 0) else {
                            error!("failed to add hms (12, 0, 0), skipping");
                            continue;
                        };
                        let Some(dt) = ndt.and_local_timezone(jst).single() else {
                            error!("failed to add jst offset, skipping");
                            continue;
                        };
                        dt.to_utc()
                    },
                    Err(e) => {
                        error!("failed to parse Ymd: {s}: {e}, skipping");
                        continue;
                    }
                };
                Some(utc)
            }
            None => None
        };

        let mut remove = vec![];
        for (i, prop) in properties.iter_mut().enumerate() {
            if prop.ends_with("、") || prop.ends_with(",") {
                prop.pop();
            }
            if prop.ends_with("：") || prop.ends_with(":") {
                prop.pop();
            }
            if prop.len() == 0 {
                remove.push(i);
            }
        }
        for i in remove.iter().rev() {
            properties.remove(*i);
        }

        let song = Song{
            title,
            member,
            published,
            links,
            properties,
        };
        songs.push(song);
    }

    Ok(songs)
}

fn main() {
    let start = time::Instant::now();
    tracing_subscriber::fmt().init();

    let songs = match scrape() {
        Ok(v) => v,
        Err(e) => panic!("{e}")
    };

    info!("{} songs found", songs.len());

    let json = match serde_json::to_string_pretty(&songs) {
        Ok(s) => s,
        Err(e) => panic!("{e}")
    };
    if let Err(e) = fs::write(DEST, json) {
        panic!("{e}");
    }
    info!("written to file: {DEST}");

    info!("done in {}milsecs", start.elapsed().as_millis());
}
