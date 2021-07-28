
use std::str::FromStr;

use clap::{Arg, App};
use ipnet::IpNet;

fn is_ipnet(value: String) -> Result<(), String> {
    IpNet::from_str(&value).map_err(|_| "Invalid IpNet.")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("cidr")
        .version("0.0.1")
        .author("John Peel <john@dgby.org>")
        .about("Aggregates a list of included and excluded CIDRs.")
        .arg(
            Arg::with_name("includes")
                .value_name("CIDR")
                .help("CIDRs to include in the list")
                .validator(is_ipnet)
                .multiple(true)
                .required(true)
        )
        .arg(
            Arg::with_name("excludes")
                .long("exclude")
                .short("e")
                .value_name("CIDR")
                .help("CIDRs to exclude from the list")
                .takes_value(true)
                .validator(is_ipnet)
                .multiple(true)
        )
        .get_matches();
    
    let exclude: Vec<IpNet> = IpNet::aggregate(&matches.values_of("excludes")
        .unwrap_or_default()
        .into_iter()
        .map(|item| IpNet::from_str(item))
        .collect::<Result<Vec<IpNet>, _>>()?);
    let include: Vec<IpNet> = IpNet::aggregate(&IpNet::aggregate(&matches.values_of("includes")
        .ok_or::<std::io::Error>(std::io::ErrorKind::InvalidInput.into())?
        .into_iter()
        .map(|item| IpNet::from_str(item))
        .collect::<Result<Vec<IpNet>, _>>()?)
        .into_iter()
        .flat_map(|possible_net| {
            let mut good_nets = vec![];
            let mut possible_queue = vec![possible_net];

            'possible: while possible_queue.len() > 0 {
                let possible_net = possible_queue.pop().unwrap();

                for bad_net in &exclude {
                    if possible_net.contains(bad_net) {
                        if possible_net.prefix_len() < bad_net.prefix_len() {
                            possible_queue.extend(possible_net.subnets(possible_net.prefix_len() + 1).unwrap());
                        }

                        continue 'possible;
                    }
                }

                good_nets.push(possible_net);
            }
            
            good_nets.into_iter()
        })
        .collect());

    println!("{}", include.iter().map(|net| net.to_string()).collect::<Vec<String>>().join(","));
    Ok(())
}
