use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio,ChildStdout,ChildStdin};
use std::path::Path;
use nix_nar::Decoder;
use std::collections::HashMap;
use std::sync::{Arc,Mutex};
#[derive(Debug)]
struct AppIcon{
    icon_name:String,
    pkg_name:String,
    extension:String,
    is_valid:bool,
}
fn main() {
    println!("process has been started");
    let mut  threads:Arc<Mutex<HashMap<i8, bool>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut  builders:Arc<Mutex<HashMap<i8, bool>>> = Arc::new(Mutex::new(HashMap::new()));
    for x in 1..100 {
        threads.lock().unwrap().insert(x, false);
        builders.lock().unwrap().insert(x, false);
    }

    let p = Command::new("mkdir").args(["-p","icons"])
    .status()
    .expect("failed to execute child");
   let  pkgs:Vec<String> =  serde_json::from_str(&match std::fs::read_to_string("./packages.json"){
        Ok(txt) => txt,
        Err(err) => "[]".to_string()
    }).expect("cant open packages");
    let mut count = 0;
    let length = pkgs.len();
    
    for pkg in pkgs{
      
        count+=1;
        let  threads = threads.clone();
        let builders = builders.clone();
        let mut thread = get_thread(&threads);
        while thread ==0{
            thread = get_thread(&threads);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        std::thread::spawn(move||{
            let re = regex::Regex::new("^nixos.").unwrap();
            let hash_re =  regex::Regex::new(r"/nix/store/(.*?)-").unwrap();
                   let pkg = re.replace(&pkg,"").to_owned();
     
        let icon:AppIcon;
        println!("thread-{}",thread);
        println!("{}/{}-->{}",count,length,pkg);
       let hash =  get_hash(&pkg,&hash_re);
            let body = reqwest::blocking::get(format!("https://cache.nixos.org/{}.ls",hash)).unwrap()
                .text().unwrap();

           if !body.contains("hicolor") && body!="404"{ // if this package does not have icons don't build
           *threads.lock().unwrap().entry(thread).or_insert(false) = false;
            return
           }
            if body.contains("hicolor"){ // if this package has icons

            icon= download_and_get_icon(&pkg,&hash,&thread);
            println!("{:?}",icon);
            if icon.is_valid{
                cp_icon(&icon);
            }
            std::fs::remove_dir_all("temp_folder-".to_owned()+&thread.to_string()).unwrap();
           }else{  // if its 404 than build and check if it has icon
            *builders.lock().unwrap().entry(thread).or_insert(true) = true;
               icon = build_and_get_icon(&pkg); // build and get icon
               if icon.is_valid{
                cp_icon(&icon);
                if !is_any_building(&builders){

                    gc();
                }
               }
               *builders.lock().unwrap().entry(thread).or_insert(false) = false;
           }
           *threads.lock().unwrap().entry(thread).or_insert(false) = false;
        });
    }

}
fn get_thread(threads:&Arc<Mutex<HashMap<i8, bool>>>)->i8{
    let mut  threads = threads.lock().unwrap();
    
   let thread =   match threads.iter().find(|(x,&y)| y==false){
        Some(x)=>*x.0,
        None=>0
    };
    if thread !=0 {
        *threads.entry(thread).or_insert(true) = true;
    }
  
    thread
}
fn is_any_building(builders:&Arc<Mutex<HashMap<i8, bool>>>)->bool{
    let mut  builders = builders.lock().unwrap();

     match builders.iter().find(|(x,&y)| y==true){
        Some(x)=>true,
        None=>false
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
       

let p = Command::new("nix-build").args(["<nixpkgs>","-A",pkg,"--no-out-link"])
.output()
.expect("failed to execute child");
let  path = std::str::from_utf8(&p.stdout).unwrap();
get_icon(path,pkg)
}

fn download_and_get_icon(pkg:&str,hash:&str,thread:&i8)->AppIcon{
       
    download_nar(hash,&thread);
    let temp_folder = "temp_folder-".to_owned()+&thread.to_string();
    let mut icon = get_icon(&temp_folder,pkg);
      let p = Command::new("readlink").arg(&icon.icon_name.trim())
    .output()
    .expect("failed to execute child");
   
    let mut  read_link = std::str::from_utf8(&p.stdout).unwrap().trim().to_owned();
    let hash_re =  regex::Regex::new(r"/nix/store/(.*?)-").unwrap();
    let remove_base =  regex::Regex::new(r"/nix/store/.*?/").unwrap();
    let mut hash = match hash_re.captures(&read_link) {
        Some(x) =>x[1].to_string(),
        None =>"None".to_string()
       };
        while !read_link.is_empty() && read_link.starts_with("/nix/store/") {
            
            std::fs::remove_dir_all(temp_folder.clone()).unwrap();
            download_nar(&hash,&thread);
            icon = AppIcon{
                icon_name: format!("{}/{}",temp_folder,remove_base.replace(&read_link,"").trim()).to_string(),
                pkg_name: pkg.to_string(),
                extension: read_link[read_link.len()- 3..].to_string(),
                is_valid:true,
            };
           
            let p = Command::new("readlink").arg(&icon.icon_name)
            .output()
            .expect("failed to execute child");
            read_link = std::str::from_utf8(&p.stdout).unwrap().to_owned();
            hash = match hash_re.captures(&read_link) {
                Some(x) =>x[1].to_string(),
                None =>"None".to_string()
               }
        };
        return icon
    }














fn gc(){
    let p = Command::new("nix-collect-garbage")
    .output()
    .expect("failed to execute child");
    let out = std::str::from_utf8(&p.stdout).unwrap();
    println!("{}",out);
}

fn get_hash(pkg:&str,re:&regex::Regex)->String{
    let p = Command::new("nix").args(["eval",&format!("nixpkgs.{}.outPath",pkg)])
    .output()
    .expect("failed to execute child");
    let out = std::str::from_utf8(&p.stdout).unwrap();



   match re.captures(out) {
    Some(x) =>x[1].to_string(),
    None =>"None".to_string()
   }

}


fn get_icon(path:&str,pkg:&str)->AppIcon{
    let path = format!("{}/share/icons/hicolor/",path.trim()); //hicolor path
    let path_struct = Path::new(&path);
    if !path_struct.exists(){
        return AppIcon{icon_name:"".to_string(),pkg_name:"".to_string(),extension:"".to_string(),is_valid:false}
    }
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


fn download_nar(hash:&str,thread:&i8){
    let  res = reqwest::blocking::get(format!("https://cache.nixos.org/{}.narinfo",hash)).unwrap()
    .text().unwrap();
    let url = res.split("\n").map(|x|x.to_string()).collect::<Vec<String>>()[1].replace("URL: ","");
    let filename = format!("temp-{}.nar.xz",thread);
    Command::new("wget").arg(&format!("https://cache.nixos.org/{}",url)).args(["-O",&filename])
    .status()
    .expect("failed to execute child");
    Command::new("unxz").arg("-f").arg(&filename)
    .status()
    .expect("failed to execute child");
    let input = std::fs::read(&filename[..filename.len()-3]).unwrap();
    let dec = Decoder::new(&input[..]).unwrap();
    dec.unpack("temp_folder-".to_owned()+&thread.to_string());
}
        // let body = reqwest::blocking::get("https://cache.nixos.org/zkjmh1llrq0ssamd5lfxyz43s09vafhr.ls").unwrap()
    // .text().unwrap();
