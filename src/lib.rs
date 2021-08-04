// pub mod operations {
//     pub fn addition(left: i32, right: i32) -> i32 {
//         left + right
//     }

//     pub fn subtraction(minuend: i32, subtraend: i32) -> i32 {
//         minuend - subtraend
//     }

//     pub fn mult(multiplier: i32, multiplicand: i32) -> i32 {
//         multiplier * multiplicand
//     }

//     pub fn division(dividend: i32, divisor: i32) -> Result<i32, &'static str> {
//         if divisor == 0 {
//             Err("Division by zero")
//         } else {
//             Ok(dividend / divisor)
//         }
//     }
// }

pub fn mult(multiplier: i32, multiplicand: i32) -> i32 {
    multiplier * multiplicand
}