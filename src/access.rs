use petgraph::Undirected;

use crate::kind::Kind;

use crate::mem_ribbon::MemRibbon;


#[non_exhaustive]
pub enum MemByte
{
    Undefined,
    OutOfBounds,
    Byte(u8),
}

impl MemByte
{

    pub fn byte(&self) -> Option<u8>
    {
        match self {
            MemByte::Byte(byte) => Some(*byte),
            _ => None,
        }
    }

    pub fn byte_mut(&mut self) -> &mut u8
    {
        *self = MemByte::Byte(0);
        match self {
            MemByte::Byte(val) => val,
            _ => unreachable!(),
        }
    }

}


pub struct PlaceValue <'kind>
{
    pub kind    : &'kind Kind <'kind>,
    pub address : usize,
}

#[derive(Debug,PartialEq,Eq)]
pub enum AccessUnit
{
    Field(String),
    Arrow(String),
    Deref,
    Index(usize),
}


impl AccessUnit
{
    pub fn op_str(&self) -> &str
    {
        match self {
            AccessUnit::Field(_) =>  ".",
            AccessUnit::Arrow(_) => "->",
            AccessUnit::Deref    =>  "*",
            AccessUnit::Index(_) => "[]",
        }
    }

}



#[derive(Debug)]
pub struct Access
{
    sequence: Vec<AccessUnit>,
}


impl Access
{
    pub fn get (base: &str) -> Self
    {
        let mut sequence = Vec::new();
        sequence.push(AccessUnit::Field(base.to_string()));
        Self {
            sequence
        }
    }

    pub fn deref(mut self) -> Self
    {
        self.sequence.push(AccessUnit::Index(0usize));
        self
    }

    pub fn index(mut self,idx: usize) -> Self
    {
        self.sequence.push(AccessUnit::Index(idx));
        self
    }

    pub fn field(mut self,fname: &str) -> Self
    {
        self.sequence.push(AccessUnit::Field(fname.to_string()));
        self
    }

    pub fn arrow(mut self, fname: &str) -> Self
    {
        self.deref().field(fname)
    }


    pub fn iter<'a>(&'a self) -> AccessIter<'a>
    {
        AccessIter {
            access:   self,
            position: 0usize,
        }
    }


}





pub struct AccessIter<'kind>
{
    access: &'kind Access,
    position: usize,
}


impl <'kind> Iterator for AccessIter <'kind>
{
    type Item = &'kind AccessUnit;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.access.sequence.get(self.position);
        self.position += 1;
        result
    }
}




pub struct AccessTrace <'kind>
{
    pub ribbon     : &'kind MemRibbon <'kind>,
    pub iter       : AccessIter <'kind>,
    pub address    : usize,
    pub field_name : String,
}




pub fn base_access_error_string(unit: &AccessUnit, type_name: String, field_name: String) -> String
{
    let op_str = unit.op_str();
    format!("Operation {op_str} undefined for {type_name} {field_name}")
}


pub fn unwind_access_error_string(original: String, type_name: String, field_name: String) -> String
{
    format!("{original} in {type_name} {field_name}")
}





#[cfg(test)]
mod test {
    use crate::access::AccessUnit;

    use super::Access;


    #[test]
    fn single(){
        let buf = Access::get("abc");
        assert_eq!(buf.sequence,vec![
            AccessUnit::Field("abc".to_string())
        ]);
    }


    #[test]
    fn field(){
        let buf = Access::get("abc")
                .field("def");
        assert_eq!(buf.sequence,vec![
            AccessUnit::Field("abc".to_string()),
            AccessUnit::Field("def".to_string())
        ]);
    }


    #[test]
    fn multi_field(){
        let buf = Access::get("abc")
                .field("def")
                .field("ghi");
        assert_eq!(buf.sequence,vec![
            AccessUnit::Field("abc".to_string()),
            AccessUnit::Field("def".to_string()),
            AccessUnit::Field("ghi".to_string())
        ]);
    }


    #[test]
    fn deref(){
        let buf = Access::get("abc").deref();
        assert_eq!(buf.sequence,vec![
            AccessUnit::Field("abc".to_string()),
            AccessUnit::Deref
        ]);
    }


    #[test]
    fn index(){
        let buf = Access::get("abc").index(2usize);
        assert_eq!(buf.sequence,vec![
            AccessUnit::Field("abc".to_string()),
            AccessUnit::Index(2usize)
        ]);
    }


    #[test]
    fn arrow(){
        let buf = Access::get("abc").arrow("def");
        assert_eq!(buf.sequence,vec![
            AccessUnit::Field("abc".to_string()),
            AccessUnit::Arrow("def".to_string())
        ]);
    }

}





// impl FromStr for AccessBuf
// {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let mut rest : Vec<AccessSegment> = Vec::new();
//         for findex, fpath in s.split('.').enumerate()
//         {
//             if findex == 0 {

//             }
//             for iindex, ipath in s.split('@').enumerate()
//             {

//             }
//         }
        
//     }
// }


