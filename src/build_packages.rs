use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio,ChildStdout,ChildStdin};
use std::path::Path;
#[derive(Debug)]
struct AppIcon{
    icon_name:String,
    pkg_name:String,
    extension:String,
    is_valid:bool,
}
fn main() {
    let p = Command::new("mkdir").args(["-p","icons"])
    .status()
    .expect("failed to execute child");
   let  pkgs:Vec<String> =  serde_json::from_str(&match std::fs::read_to_string("./build_packages.json"){
        Ok(txt) => txt,
        Err(err) => "[]".to_string()
    }).expect("cant open packages");
    let mut count = 0;
    let length = pkgs.len();
    let re = regex::Regex::new("^nixos.").unwrap();
    let hash_re =  regex::Regex::new(r"/nix/store/(.*?)-").unwrap();

    for pkg in pkgs{
        let icon:AppIcon;
        let pkg = re.replace(&pkg,"");
        count+=1;
        println!("{}/{}-->{}",count,length,pkg);

               icon = build_and_get_icon(&pkg); // build and get icon
                println!("{:?}",icon);
               if icon.is_valid{
                cp_icon(&icon);
               }
      let pkg_out = Path::new("./pkg_out");
          if pkg_out.exists(){
            if pkg_out.is_dir(){
            std::fs::remove_dir_all("pkg_out".to_owned()).unwrap();
            } else{
              std::fs::remove_file("pkg_out".to_owned()).unwrap();
            }
          
          
          };
   
            gc();
           }
   
       
    }




fn folders(dir: &Path) -> Result<Vec<String>, std::io::Error> {
    Ok(std::fs::read_dir(dir)?
        .into_iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().path().is_dir()) // Get rid of Err variants for Result<DirEntry>
        .map(|r| r.unwrap().path().display().to_string().split("/").last().unwrap().to_string()) // This is safe, since we only have the Ok variants
        .collect())
}

fn cp_icon(icon:&AppIcon){
    let p = Command::new("cp").args([icon.icon_name.trim(),&format!("./icons/{}.{}",icon.pkg_name,icon.extension)])
    .status()
    .expect("failed to execute child");
}



fn build_and_get_icon(pkg:&str)->AppIcon{
      

let p = Command::new("nix-build").args(["<nixpkgs>","-A",&format!("{}.src",pkg),"--no-out-link"])
.output()
.expect("failed to execute child");

let  path = std::str::from_utf8(&p.stdout).unwrap().trim();
    
let path_struct = Path::new(&path);
if path_struct.is_dir(){
   return get_icon(path,pkg);
}else if path.ends_with(".deb"){
  println!("-->>{}",path);
 let _p = Command::new("dpkg").args(["-x",path,"pkg_out"])
    .output()
    .expect("failed to execute child");
    return get_icon("./pkg_out",pkg);
}else if path.contains(".tar"){
   println!("-->>{}",path);
 let p = Command::new("tar").args(["-xvf",path])
    .output()
    .expect("failed to execute child");
  let  path = std::str::from_utf8(&p.stdout).unwrap().trim();
  let lines = path.split("\n").collect::<Vec<&str>>();
  std::fs::rename(lines[0],"pkg_out").unwrap();
    return get_icon("./pkg_out",pkg);
};
    AppIcon{icon_name:"".to_string(),pkg_name:"".to_string(),extension:"".to_string(),is_valid:false}

}





fn gc(){
    let p = Command::new("nix-collect-garbage")
    .output()
    .expect("failed to execute child");
    let _out = std::str::from_utf8(&p.stdout).unwrap();
    println!("collecting garbage...");
}


fn get_icon(dir_path:&str,pkg:&str)->AppIcon{
    let path = format!("{}/share/icons/hicolor/",dir_path.trim()); //hicolor path
    let path_struct = Path::new(&path);
    if path_struct.exists(){
    let p = Command::new("find").args([&path,"-name","*.svg","-print","-quit"])
    .output()
    .expect("failed to execute child"); // check if svg exists
    let svg =  std::str::from_utf8(&p.stdout).unwrap();
    
    if !svg.is_empty(){
        println!("{}",svg);
        return  AppIcon{icon_name:svg.to_string(),pkg_name:pkg.to_string(),extension:"svg".to_string(),is_valid:true};
    };
    
    
    let mut sizes = folders(path_struct).unwrap();
    
    
    let size = get_resolution(sizes);
    println!("{}",size);
    let p = Command::new("find").args([&format!("{}{}",path,size),"-name","*.png","-print","-quit"])
    .output()
    .expect("failed to execute child"); // check if svg exists
    let png =  std::str::from_utf8(&p.stdout).unwrap();
    println!("{}",png);
    if !png.is_empty(){
        return  AppIcon{icon_name:png.to_string(),pkg_name:pkg.to_string(),extension:"png".to_string(),is_valid:true};
       
    };


    let p = Command::new("find").args([&format!("{}{}",path,size),"-name","*.gif","-print","-quit"])
    .output()
    .expect("failed to execute child"); // check if svg exists
    let gif =  std::str::from_utf8(&p.stdout).unwrap();
    println!("{}",gif);
    if !gif.is_empty(){
        return  AppIcon{icon_name:gif.to_string(),pkg_name:pkg.to_string(),extension:"gif".to_string(),is_valid:true};
       
    };

    
    };
    let p = Command::new("find").args([dir_path,"-name","*.png","-or","-name","*.gif","-or","-name","*.svg"])
    .output()
    .expect("failed to execute child"); // check if svg exists
  let re_size =  regex::Regex::new(r"(\d+)").unwrap();
    let mut  icons:Vec<String> =  std::str::from_utf8(&p.stdout).unwrap().split("\n").filter(|x|!x.is_empty()).map(|x| x.to_string()).collect();
    icons.sort_by(|a,b|{
      let by_name = b.split(r"/").last().unwrap().contains(&pkg).cmp(&a.split(r"/").last().unwrap().contains(&pkg));
      let by_logo = b.contains("logo").cmp(&a.contains("logo"));
      let a_size =  match re_size.captures(&a) {
        Some(x) =>x[1].parse().unwrap(),
        None =>0
       };
      let b_size =  match re_size.captures(&b) {
        Some(x) =>x[1].parse().unwrap(),
        None =>0
       };
      let by_size = b_size.cmp(&a_size);
      if by_name != std::cmp::Ordering::Equal{
        return by_name;
      }else if by_logo != std::cmp::Ordering::Equal{
        return by_logo;
      }else if by_size != std::cmp::Ordering::Equal{
        return by_size
      }
      std::cmp::Ordering::Equal
    });
   
    if !icons.is_empty(){
      
       return  AppIcon{icon_name:icons[0].to_string(),pkg_name:pkg.to_string(),extension:icons[0][icons[0].len()- 3..].to_string(),is_valid:true};
    };



    return AppIcon{icon_name:"".to_string(),pkg_name:"".to_string(),extension:"".to_string(),is_valid:false}
}










fn get_resolution(mut sizes:Vec<String>)->String{

    sizes.sort_by(|a,b|{
        let a_width:i32 = match a.split("x").last().unwrap().parse() {
            Ok(x) => x,
            Err(_) =>0
        };
        let b_width:i32 =match  b.split("x").last().unwrap().parse() {
            Ok(x) => x,
            Err(_) =>0
        };
        a_width.cmp(&b_width)
    });

    println!("{:?}",sizes);
    if sizes.contains(&"128x128".to_string()){
        "128x128".to_string()
    }else{
        let index = match sizes.iter().position(|r| match r.split("x").last().unwrap().parse(){Ok(x)=>x,Err(_)=>0} > 128){
            Some(x)=>x,
            None => if sizes.len() > 1 {sizes.len()-1} else {0}
        };
        sizes[index].to_string()
    }

}
