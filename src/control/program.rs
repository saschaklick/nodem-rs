use virtmach::VirtMach;
use crate::{media::Media, control::{ Control, IControl }};

#[repr(u8)]
enum Ret {
    Ok = 0,
    #[cfg(feature = "vm")]
    InvalidProgram = 1,
    InvalidInstruction = 2
}

impl IControl for VirtMach<'_> {    
    fn process_line(&mut self, line: &str, _media: &Media, res: &mut dyn core::fmt::Write) -> (bool, core::fmt::Result) {                                    
        let prefix = "vm.";
        if line.starts_with(prefix){
            let line = &line[prefix.len()..];
            let mut ret = Ret::Ok;
            match line {
                "pau" => { self.pause(); }
                "rst" => { self.reset(); }
                "unl" => { self.unload(); }
                "run" => { todo!(); /*self.run(1024, &mut []);*/ }
                "stp" => { todo!(); /*self.run(1, &mut []);*/ }                
                #[cfg(feature = "inspect")]
                "ins" => { ins(self, res).expect(""); }
                #[cfg(feature = "inspect")]
                "reg" => { reg(self, res).expect(""); }
                #[cfg(feature = "inspect")]
                "mem" => { mem(self, res).expect(""); }                
                _ => {
                    let split = line.split_once('=').unwrap_or(("", ""));
                    match split.0.trim() {                
                        #[cfg(feature = "vm")]
                        "prg" => {
                            let value_res = split.1.trim().parse::<u8>();
                            match value_res {
                                Ok(idx) => { self.load_program(_media.get_program(idx)); },
                                Err(_) => { ret = Ret::InvalidProgram; }
                            }
                        },            
                        _ => { ret = Ret::InvalidInstruction; }
                    }
                }
            }            
            Control::send_result(prefix, ret as u8, res)
        }else{
            (false, Ok(()))
        }
    }          
}

fn reg(vm: &VirtMach, res: &mut dyn core::fmt::Write) -> core::fmt::Result{    
    for (i, atom) in vm.registers.iter().enumerate() {                
        res.write_fmt(format_args!("{}", atom)).expect("");
        res.write_str(if i % 8 == 7 || i == vm.registers.len() - 1 { "\r\n" } else { "," }).expect("");          
    }
    Ok(())
}

fn mem(vm: &VirtMach, res: &mut dyn core::fmt::Write) -> core::fmt::Result{    
    for (i, atom) in vm.memory.iter().enumerate() {                
        res.write_fmt(format_args!("{}", atom)).expect("");
        res.write_str(if i % 8 == 7 || i == vm.memory.len() - 1 { "\r\n" } else { "," }).expect("");        
    }
    Ok(())
}

fn ins(vm: &VirtMach, res: &mut dyn core::fmt::Write) -> core::fmt::Result{        
    vm.inspect(res).expect("");
    Ok(())
}