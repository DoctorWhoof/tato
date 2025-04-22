mod assets;
use assets::*;
use tato::Anim;

fn main(){
    println!("size of anim: {}", size_of::<Anim<4,6>>());
    println!("FG_PALETTE: {:?}", FG_PALETTE);
}
