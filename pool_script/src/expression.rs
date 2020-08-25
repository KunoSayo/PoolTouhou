use std::convert::TryFrom;
use std::io::{Error, ErrorKind};

use crate::context::Context;
use crate::pool_script::Compile;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Operator {
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    EQ,
    NEQ,
    GT,
    LT,
    GE,
    LE,
    LeftB,
    RightB,
}


impl Operator {
    fn get_priority(&self) -> u8 {
        match self {
            Operator::ADD => 1,
            Operator::SUB => 1,
            Operator::MUL => 2,
            Operator::DIV => 2,
            Operator::MOD => 2,
            Operator::LeftB => 0,
            Operator::RightB => 0,
            _ => 1
        }
    }

    fn operate(&self, v1: f32, v2: f32) -> f32 {
        match self {
            Operator::ADD => v1 + v2,
            Operator::SUB => v1 - v2,
            Operator::MUL => v1 * v2,
            Operator::DIV => v1 / v2,
            Operator::MOD => v1 % v2,
            _ => panic!("Not supported {:?}", self)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ExpressionElement {
    CONST(f32),
    GAME(u8),
    DATA(u8),
    STACK(u8),
    OP(Operator),
}


impl Compile for Operator {
    fn flush(&self, binary: &mut Vec<u8>) -> Result<(), Error> {
        match self {
            Operator::ADD => binary.push(21),
            Operator::SUB => binary.push(22),
            Operator::MUL => binary.push(23),
            Operator::DIV => binary.push(24),
            Operator::MOD => binary.push(25),
            _ => { return Err(Error::new(ErrorKind::InvalidData, "[parse expression]expected operator but found : ".to_owned() + stringify!(self))); }
        }
        Ok(())
    }
}

impl Compile for ExpressionElement {
    fn flush(&self, binary: &mut Vec<u8>) -> Result<(), Error> {
        match self {
            ExpressionElement::CONST(value) => {
                binary.push(0);
                for byte in value.to_be_bytes().iter() {
                    binary.push(*byte);
                }
            }
            ExpressionElement::GAME(index) => {
                binary.push(1);
                binary.push(*index);
            }
            ExpressionElement::DATA(index) => {
                binary.push(2);
                binary.push(*index);
            }
            ExpressionElement::STACK(index) => {
                binary.push(3);
                binary.push(*index);
            }
            ExpressionElement::OP(op) => {
                op.flush(binary)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Expression {
    tree: Vec<ExpressionElement>,
    op: Vec<Operator>,
}

impl Expression {
    fn push_operator(&mut self, op: Operator) {
        match op {
            Operator::LeftB => {
                self.op.push(op);
            }
            Operator::RightB => {
                while let Some(old_op) = self.op.pop() {
                    if old_op == Operator::LeftB {
                        break;
                    }
                    self.push_tree(ExpressionElement::OP(old_op));
                }
            }
            _ => {
                while self.op.len() > 0 {
                    let top_op = self.op[self.op.len() - 1];
                    if top_op.get_priority() >= op.get_priority() {
                        self.push_tree(ExpressionElement::OP(top_op));
                        self.op.pop();
                    } else {
                        break;
                    }
                }
                self.op.push(op);
            }
        }
    }

    fn push_tree(&mut self, value: ExpressionElement) {
        if let ExpressionElement::OP(op) = value {
            let len = self.tree.len();
            if len > 1 {
                let value1 = self.tree[len - 1];
                let value0 = self.tree[len - 2];
                if let (ExpressionElement::CONST(c1), ExpressionElement::CONST(c2)) = (value0, value1) {
                    self.tree.pop();
                    self.tree.pop();
                    self.tree.push(ExpressionElement::CONST(op.operate(c1, c2)));
                    return;
                }
            }
        }
        self.tree.push(value);
    }
}

impl Compile for Expression {
    fn flush(&self, binary: &mut Vec<u8>) -> Result<(), Error> {
        for value in &self.tree {
            if let ExpressionElement::OP(_) = value {} else {
                binary.push(3);
            }
            value.flush(binary)?
        }
        Ok(())
    }
}

pub fn try_parse_expression(raw_str: &str, context: &Context) -> Result<Expression, Error> {
    let s = raw_str.replace(" ", "");
    let mut begin = 0;
    let mut expression = Expression {
        tree: vec![],
        op: vec![],
    };

    let mut index = 0;
    let mut parsing_value = true;
    while index < s.len() {
        if let Ok(op) = Operator::try_from(&s[index..index + 1]) {
            if parsing_value && index == begin && op == Operator::SUB {
                index += 1;
                while s[index..index + 1].chars().next().unwrap().is_ascii_digit() {
                    index += 1;
                }
                if begin != index {
                    if let Ok(value) = context.parse_value(&s[begin..index]) {
                        expression.push_tree(value);
                    }
                }
                parsing_value = false;
                continue;
            } else {
                if parsing_value && begin != index {
                    if let Ok(value) = context.parse_value(&s[begin..index]) {
                        expression.push_tree(value);
                    }
                }
                expression.push_operator(op);
                parsing_value = true;
                begin = index + 1;
            }
        }
        index += 1;
    }

    if parsing_value && begin != index {
        let value = context.parse_value(&s[begin..s.len()])?;
        expression.push_tree(value);
    }
    while let Some(op) = expression.op.pop() {
        expression.push_tree(ExpressionElement::OP(op));
    }

    Ok(expression)
}

impl TryFrom<&str> for Operator {
    type Error = Error;

    fn try_from(str: &str) -> Result<Self, Self::Error> {
        match str {
            "+" => Ok(Operator::ADD),
            "-" => Ok(Operator::SUB),
            "*" => Ok(Operator::MUL),
            "/" => Ok(Operator::DIV),
            "%" => Ok(Operator::MOD),
            "(" => Ok(Operator::LeftB),
            ")" => Ok(Operator::RightB),
            _ => Err(Error::new(ErrorKind::InvalidData, "[parse expression]expected operator but found : ".to_owned() + str))
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::context::Context;
    use crate::expression::{Expression, ExpressionElement, Operator, try_parse_expression};

    #[test]
    fn parse_expression() {
        let mut map = HashMap::new();
        map.insert("a0".to_string(), 0);
        map.insert("a1".to_string(), 1);
        map.insert("a2".to_string(), 2);
        let mut context = Context::new(&map);
        context.push_name("b0");
        context.push_stack();
        context.push_name("valid");
        context.pop_stack();
        context.push_name("b1");

        let mut value = try_parse_expression("1-2+3", &context).unwrap();
        assert_eq!(value.tree.pop().unwrap(), ExpressionElement::CONST(2.0));
        let mut value = try_parse_expression("-1-2+3", &context).unwrap();
        assert_eq!(value.tree.pop().unwrap(), ExpressionElement::CONST(0.0));
        let mut value = try_parse_expression("1+2*3", &context).unwrap();
        assert_eq!(value.tree.pop().unwrap(), ExpressionElement::CONST(7.0));
        let mut value = try_parse_expression("1 * ( 2 + 3 )", &context).unwrap();
        assert_eq!(value.tree.pop().unwrap(), ExpressionElement::CONST(5.0));
        let mut value = try_parse_expression("3/2", &context).unwrap();
        assert_eq!(value.tree.pop().unwrap(), ExpressionElement::CONST(1.5));
        let mut value = try_parse_expression("(1+2)%2", &context).unwrap();
        assert_eq!(value.tree.pop().unwrap(), ExpressionElement::CONST(1.0));
        let mut value = try_parse_expression("a1 * ( b1 + a2 )", &context).unwrap();
        assert_eq!(value.tree, vec![ExpressionElement::DATA(1), ExpressionElement::STACK(1), ExpressionElement::DATA(2), ExpressionElement::OP(Operator::ADD), ExpressionElement::OP(Operator::MUL)]);
    }
}