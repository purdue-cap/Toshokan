use std::collections::HashMap;
use std::io::{BufRead, BufReader, Error, Cursor};
use std::iter::repeat;
use std::path::Path;
use std::fs::File;
use regex::Regex;

pub struct LogAnalyzer<'n, 's> {
    c_e_names: &'n [&'s str],
    lookup_map: HashMap<&'s str, usize>
}

impl<'n, 's> LogAnalyzer<'n, 's> {
    pub fn new(names: &'n [&'s str]) -> Self{
        LogAnalyzer {
            c_e_names: names,
            lookup_map: names.iter().enumerate()
                        .map(|(index, value)| (*value, index)).collect()
        }
    }

    pub fn read_c_e_s<R: BufRead>(&self, mut reader: R) -> Result<Vec<isize>, Error> {
        let mut buffer = String::new();
        let mut c_e_s : Vec<Option<isize>> = repeat(None).take(self.c_e_names.len()).collect();
        let mut in_find = false;
        loop {
            if reader.read_line(&mut buffer)? == 0 {
                break;
            }
            match buffer.as_str() {
                "BEG FIND\n" => {
                    in_find = true;
                },
                "END FIND\n" => {
                    in_find = false;
                },
                line => {
                    if in_find {
                        let pattern = Regex::new(r" input (\w+) has value \d+= \((\d+)\)")
                                    .expect("Hard coded regex should be correct");
                        let captures = pattern.captures(&line[..line.len()-1]);
                        let parse_result =
                                captures.map(|cap|
                                    cap.get(1).and_then(|name| cap.get(2).map(
                                        |value| (name.as_str(), value.as_str())
                                    )).and_then(|(name, value)|
                                        Some((name, value.parse::<isize>().ok()?))
                                    )
                                ).flatten();
                        if let Some((name, value)) = parse_result {
                            self.lookup_map.get(name)
                                .and_then(|index|{
                                    *c_e_s.get_mut(*index)? = Some(value);
                                    Some(())
                                });
                        }
                    }
                }

            }
            buffer.clear();
        }
        c_e_s.into_iter().collect::<Option<_>>().ok_or(Error::new(std::io::ErrorKind::Other, "Missing C.E.(s)"))
    }
    
    pub fn read_c_e_s_from_str<S: AsRef<str>>(&self, logs: S) -> Result<Vec<isize>, Error> {
        let cursor = Cursor::new(logs.as_ref());
        self.read_c_e_s(cursor)
    }

    pub fn read_c_e_s_from_file<P: AsRef<Path>>(&self, logs_file_name: P) -> Result<Vec<isize>, Error> {
        let logs_file = File::open(logs_file_name)?;
        self.read_c_e_s(BufReader::new(logs_file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn reads_c_e_s_from_str() -> Result<(), Box<dyn Error>> {
        let names = ["p_3_6_0"];
        let logs = r#"
SKETCH version 1.7.5
Benchmark = verification.sk
[SATBackend] ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
[SATBackend] MAX LOOP UNROLLING = 8
[SATBackend] MAX FUNC INLINING  = 5
[SATBackend] After prog.accept(partialEval)
[SATBackend] OFILE = null
resolved cegis to path /usr/share/sketchsynth/bin/cegis
[SATBackend] Launching: /usr/share/sketchsynth/bin/cegis --bnd-inbits 3 --boundmode CALLNAME --verbosity 3 --print-version -simiters 4 --print-cex --assumebcheck --bnd-inline-amnt 5 -angelictupledepth 1 -srctupledepth 2 -sprandbias 1 -o /home/hkj/.sketch/tmp/verification.sk/solution0-0 /home/hkj/.sketch/tmp/verification.sk/input0.tmp 
Overriding inputs with 3
boundmode = CALLNAME
assuming  bounds checks
SOLVER RAND SEED = 1586513233
Reading SKETCH Program in File /home/hkj/.sketch/tmp/verification.sk/input0.tmp
* before  EVERYTHING: main__WrapperNospec::SPEC nodes = 1	 main__Wrapper::SKETCH nodes = 25
 INBITS = 3
 CBITS  = 5
 input_ints = 1 	 input_bits = 0
* Final Problem size: Problem nodes = 270
  # OF CONTROLS:    0
 control_ints = 0 	 control_bits = 0
inputSize = 4	ctrlSize = 0
Random seeds = 1
BEG CHECK
 * After optims it became = 250 was 270
Assert at verification.sk:64 (0)
END CHECK
********  0	ftime= 0	ctime= 0
BEG FIND
Level 1  intsize = 2
 input p_3_6_0 has value 270= (2)
 * After optims it became = 2 was 270
  UNSATISFIABLE ASSERTION Assert at verification.sk:64 (0)
  UNSATISFIABLE ASSERTION Assert at verification.sk:64 (0)
Problem became UNSAT.1= ASSERT (0) : Assert at verification.sk:64 (0)
* TIME TO ADD INPUT :  0 ms 
  UNSATISFIABLE ASSERTION Assert at verification.sk:64 (0)
END FIND
******** FAILED ********
* FIND TIME:  0 ms 
* CHECK TIME:  0 ms 
 *FAILED IN 1 iterations.
 *FIND TIME 0 CHECK TIME 0 TOTAL TIME 0
VALUES 
RESULT = 1  
**ROUND 0 : 0 Round time:  0.002 ms 
RNDDEG = -1
SUMMRY 
 charness = 0
POST-SUMMRY 
TRAIL: 
NOTOK
ALLRESET
return 1
[SATBackend] Stats for last run:
      [solution stats]
      successful? ---------------------> false
      elapsed time (s) ----------------> 1.03
      model building time (s) ---------> -0.001
      solution time (s) ---------------> -0.001
      max memory usage (MiB) ----------> -9.536743E-7
      [SAT-specific solution stats]
      initial number of nodes ---------> 25
      number of nodes after opts ------> 270
      number of controls --------------> 0
      total number of control bits ----> 0

[SATBackend] Solver exit value: 1
Total time = 1222
        "#;
        let analyzer = LogAnalyzer::new(&names);
        let result = analyzer.read_c_e_s_from_str(&logs)?;
        assert_eq!(result, vec![2]);
        Ok(())
    }
}

