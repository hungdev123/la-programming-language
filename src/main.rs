// =============================
// Lá v1.0
// =============================

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;

#[derive(Debug, Clone)]
enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    List(Vec<Value>),
    Null,
}

#[derive(Debug)]
enum Flow {
    None,
    Break,
    Return(Value),
}

struct Context {
    variables: HashMap<String, Value>,
    functions: HashMap<String, String>,
}

fn main() {

    let args: Vec<String> =
        env::args().collect();

    if args.len() < 2 {

        println!("Dùng: la <file.la>");
        return;
    }

    let path = &args[1];

    let source =
        fs::read_to_string(path)
        .expect("Không đọc được file");

    run(&source);
}

fn run(source: &str) {

    let mut vars =
        HashMap::new();

    vars.insert(
        "mãi".to_string(),
        Value::Number(
            f64::INFINITY
        )
    );

    vars.insert(
        "đúng".to_string(),
        Value::Bool(true)
    );

    vars.insert(
        "sai".to_string(),
        Value::Bool(false)
    );

    let mut context =
        Context {

            variables: vars,

            functions:
                HashMap::new(),
        };

    let lines: Vec<&str> =
        source.lines().collect();

    execute_lines(
        &lines,
        &mut context
    );
}

fn execute_lines(
    lines: &[&str],
    context: &mut Context,
) -> Flow {

    let mut i = 0;

    while i < lines.len() {

        let line =
            lines[i].trim();

        // =====================
        // skip
        // =====================

        if line.is_empty()
            ||
           line.starts_with("#")
        {
            i += 1;
            continue;
        }

        // =====================
        // in
        // =====================

        if line.starts_with("in ") {

            let expr =
                line
                .trim_start_matches("in ")
                .trim();

            print_expr(
                expr,
                &context.variables
            );
        }

        // =====================
        // nhập
        // =====================

        else if line == "nhập" {

            input(None);
        }

        else if line.starts_with("nhập ") {

            let prompt =
                line
                .trim_start_matches("nhập ")
                .trim();

            if let Some(Value::String(text))
                = eval_value(
                    prompt,
                    &context.variables
                )
            {
                input(Some(&text));
            }
        }

        // =====================
        // đặt
        // =====================

        else if line.starts_with("đặt ") {

            let rest =
                line
                .trim_start_matches("đặt ")
                .trim();

            if let Some((name, value))
                = rest.split_once('=')
            {
                let name =
                    name.trim();

                let value =
                    value.trim();

                // nhập
                if value.starts_with("nhập") {

                    let rest =
                        value
                        .trim_start_matches("nhập")
                        .trim();

                    let result =
                        if rest.is_empty()
                        {
                            input(None)
                        }

                        else {

                            if let Some(
                                Value::String(prompt)
                            )
                            = eval_value(
                                rest,
                                &context.variables
                            )
                            {
                                input(
                                    Some(&prompt)
                                )
                            }

                            else {

                                String::new()
                            }
                        };

                    context.variables.insert(
                        name.to_string(),
                        Value::String(result)
                    );
                }

                else if let Some(parsed)
                    = eval_value(
                        value,
                        &context.variables
                    )
                {
                    context.variables.insert(
                        name.to_string(),
                        parsed
                    );
                }
            }
        }

        // =====================
        // thêm
        // =====================

        else if line.starts_with("thêm ") {

            let rest =
                line
                .trim_start_matches("thêm ")
                .trim();

            // thêm x tại y ở z
            if rest.contains(" ở ") {

                if let Some((left, index_part))
                    = rest.split_once(" ở ")
                {
                    if let Some((value_expr, list_name))
                        = left.split_once("tại")
                    {
                        let value_expr =
                            value_expr.trim();

                        let list_name =
                            list_name.trim();

                        let index =
                            eval_math(
                                index_part.trim(),
                                &context.variables
                            )
                            .unwrap_or(0.0)
                            as usize;

                        if let Some(value)
                            = eval_value(
                                value_expr,
                                &context.variables
                            )
                        {
                            if let Some(
                                Value::List(list)
                            )
                            = context
                            .variables
                            .get_mut(list_name)
                            {
                                if index <= list.len() {

                                    list.insert(
                                        index,
                                        value
                                    );
                                }

                                else {

                                    println!(
                                        "Vị trí vượt quá danh sách"
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // thêm bình thường
            else {

                if let Some((value_expr, list_name))
                    = rest.split_once("tại")
                {
                    let value_expr =
                        value_expr.trim();

                    let list_name =
                        list_name.trim();

                    if let Some(value)
                        = eval_value(
                            value_expr,
                            &context.variables
                        )
                    {
                        if let Some(
                            Value::List(list)
                        )
                        = context
                        .variables
                        .get_mut(list_name)
                        {
                            list.push(value);
                        }
                    }
                }
            }
        }

        // =====================
        // tạo
        // =====================

        else if line.starts_with("tạo ") {

            let func_name =
                line
                .trim_start_matches("tạo ")
                .trim();

            let (
                block,
                new_i
            ) = collect_block(
                lines,
                i + 1
            );

            context.functions.insert(
                func_name.to_string(),
                block.join("\n")
            );

            i = new_i;
        }

        // =====================
        // gọi hàm
        // =====================

        else if context
            .functions
            .contains_key(line)
        {
            let source =
                context
                .functions
                .get(line)
                .unwrap()
                .clone();

            let func_lines:
                Vec<&str>
                = source.lines().collect();

            execute_lines(
                &func_lines,
                context
            );
        }

        // =====================
        // trả
        // =====================

        else if line.starts_with("trả ") {

            let expr =
                line
                .trim_start_matches("trả ")
                .trim();

            if let Some(value)
                = eval_value(
                    expr,
                    &context.variables
                )
            {
                return Flow::Return(value);
            }

            return Flow::Return(
                Value::Null
            );
        }

        // =====================
        // dừng
        // =====================

        else if line == "dừng" {

            return Flow::Break;
        }

        // =====================
        // nếu
        // =====================

        else if line.starts_with("nếu ") {

            let condition =
                line
                .trim_start_matches("nếu ")
                .trim();

            let (
                block,
                new_i
            ) = collect_if_block(
                lines,
                i + 1
            );

            let mut branches:
                Vec<(String, Vec<String>)>
                = Vec::new();

            let mut else_block:
                Vec<String>
                = Vec::new();

            let mut current_condition =
                condition.to_string();

            let mut current_block:
                Vec<String>
                = Vec::new();

            let mut in_else = false;

            for inner in block {

                let inner =
                    inner.trim();

                if inner.starts_with("nếu ")
                    &&
                   inner != "nếu không"
                    &&
                   !in_else
                {
                    branches.push((
                        current_condition.clone(),
                        current_block.clone()
                    ));

                    current_condition =
                        inner
                        .trim_start_matches("nếu ")
                        .trim()
                        .to_string();

                    current_block.clear();

                    continue;
                }

                if inner == "nếu không" {

                    branches.push((
                        current_condition.clone(),
                        current_block.clone()
                    ));

                    current_block.clear();

                    in_else = true;

                    continue;
                }

                if in_else {

                    else_block.push(
                        inner.to_string()
                    );
                }

                else {

                    current_block.push(
                        inner.to_string()
                    );
                }
            }

            if !current_block.is_empty()
                &&
               !in_else
            {
                branches.push((
                    current_condition.clone(),
                    current_block.clone()
                ));
            }

            let mut executed =
                false;

            for (condition, block)
                in branches
            {
                if let Some(result)
                    = eval_condition(
                        &condition,
                        &context.variables
                    )
                {
                    if result {

                        let refs:
                            Vec<&str>
                            = block
                            .iter()
                            .map(|s| s.as_str())
                            .collect();

                        let flow =
                            execute_lines(
                                &refs,
                                context
                            );

                        match flow {

                            Flow::Break => {
                                return Flow::Break;
                            }

                            Flow::Return(v) => {
                                return Flow::Return(v);
                            }

                            _ => {}
                        }

                        executed = true;

                        break;
                    }
                }
            }

            if !executed
                &&
               !else_block.is_empty()
            {
                let refs:
                    Vec<&str>
                    = else_block
                    .iter()
                    .map(|s| s.as_str())
                    .collect();

                let flow =
                    execute_lines(
                        &refs,
                        context
                    );

                match flow {

                    Flow::Break => {
                        return Flow::Break;
                    }

                    Flow::Return(v) => {
                        return Flow::Return(v);
                    }

                    _ => {}
                }
            }

            i = new_i;
        }

        // =====================
        // lặp
        // =====================

        else if line.starts_with("lặp ") {

            let repeat_expr =
                line
                .trim_start_matches("lặp ")
                .trim();

            let count: usize;

            let mut condition:
                Option<String> = None;

            if let Some((left, right))
                = repeat_expr
                .split_once("đến")
            {
                count =
                    eval_math(
                        left.trim(),
                        &context.variables
                    )
                    .unwrap_or(0.0)
                    as usize;

                condition =
                    Some(
                        right.trim()
                        .to_string()
                    );
            }

            else {

                count =
                    eval_math(
                        repeat_expr,
                        &context.variables
                    )
                    .unwrap_or(0.0)
                    as usize;
            }

            let (
                block,
                new_i
            ) = collect_block(
                lines,
                i + 1
            );

            for _ in 0..count {

                if let Some(cond)
                    = &condition
                {
                    if let Some(result)
                        = eval_condition(
                            cond,
                            &context.variables
                        )
                    {
                        if result {
                            break;
                        }
                    }
                }

                let refs:
                    Vec<&str>
                    = block
                    .iter()
                    .map(|s| s.as_str())
                    .collect();

                let flow =
                    execute_lines(
                        &refs,
                        context
                    );

                match flow {

                    Flow::Break => {
                        break;
                    }

                    Flow::Return(v) => {
                        return Flow::Return(v);
                    }

                    _ => {}
                }
            }

            i = new_i;
        }

        i += 1;
    }

    Flow::None
}

fn collect_block(
    lines: &[&str],
    start: usize,
) -> (Vec<String>, usize) {

    let mut block =
        Vec::new();

    let mut depth = 1;

    let mut i = start;

    while i < lines.len() {

        let line =
            lines[i].trim();

        if line.starts_with("nếu ")
            ||
           line.starts_with("lặp ")
            ||
           line.starts_with("tạo ")
        {
            depth += 1;
        }

        if line == "kết" {

            depth -= 1;

            if depth == 0 {
                break;
            }
        }

        block.push(
            line.to_string()
        );

        i += 1;
    }

    (block, i)
}

fn collect_if_block(
    lines: &[&str],
    start: usize,
) -> (Vec<String>, usize) {

    let mut block =
        Vec::new();

    let mut depth = 1;

    let mut i = start;

    while i < lines.len() {

        let line =
            lines[i].trim();

        if (
            line.starts_with("nếu ")
            &&
            line != "nếu không"
        )
        ||
           line.starts_with("lặp ")
        ||
           line.starts_with("tạo ")
        {
            depth += 1;
        }

        if line == "kết" {

            depth -= 1;

            if depth == 0 {
                break;
            }
        }

        block.push(
            line.to_string()
        );

        i += 1;
    }

    (block, i)
}

fn input(
    prompt: Option<&str>
) -> String {

    if let Some(text)
        = prompt
    {
        print!("{}", text);

        use std::io::Write;

        io::stdout()
            .flush()
            .unwrap();
    }

    let mut buffer =
        String::new();

    io::stdin()
        .read_line(&mut buffer)
        .unwrap();

    buffer
        .trim()
        .to_string()
}

fn print_expr(
    expr: &str,
    variables: &HashMap<String, Value>,
) {

    if expr.starts_with("liệt kê ") {

        let name =
            expr
            .trim_start_matches("liệt kê ")
            .trim();

        if let Some(
            Value::List(list)
        )
        = variables.get(name)
        {
            println!("{:?}", list);
            return;
        }
    }

    if expr.starts_with("độ dài của ") {

        let name =
            expr
            .trim_start_matches("độ dài của ")
            .trim();

        if let Some(
            Value::List(list)
        )
        = variables.get(name)
        {
            println!(
                "{}",
                list.len()
            );

            return;
        }
    }

    if let Some(value)
        = eval_value(
            expr,
            variables
        )
    {
        match value {

            Value::String(s) => {
                println!("{}", s);
            }

            Value::Number(n) => {
                println!("{}", n);
            }

            Value::Bool(b) => {
                println!("{}", b);
            }

            Value::List(list) => {
                println!("{:?}", list);
            }

            Value::Null => {
                println!("null");
            }
        }

        return;
    }

    println!(
        "Không hiểu biểu thức: {}",
        expr
    );
}

fn eval_value(
    expr: &str,
    variables: &HashMap<String, Value>,
) -> Option<Value> {

    // list
    if expr.starts_with('[')
        &&
       expr.ends_with(']')
    {
        let inner =
            &expr[1..expr.len()-1];

        let mut list =
            Vec::new();

        for part in inner.split(',') {

            let item =
                part.trim();

            if item.is_empty() {
                continue;
            }

            if let Some(value)
                = eval_value(
                    item,
                    variables
                )
            {
                list.push(value);
            }
        }

        return Some(
            Value::List(list)
        );
    }

    // string
    if expr.starts_with('"')
        &&
       expr.ends_with('"')
        &&
       !expr.contains('+')
    {
        return Some(
            Value::String(
                expr[1..expr.len()-1]
                .to_string()
            )
        );
    }

    // variable
    if let Some(value)
        = variables.get(expr)
    {
        return Some(
            value.clone()
        );
    }

    // bool
    if expr == "đúng" {
        return Some(
            Value::Bool(true)
        );
    }

    if expr == "sai" {
        return Some(
            Value::Bool(false)
        );
    }

    // math
    if let Some(result)
        = eval_math(
            expr,
            variables
        )
    {
        return Some(
            Value::Number(result)
        );
    }

    // string concat
    if let Some(text)
        = eval_string(
            expr,
            variables
        )
    {
        return Some(
            Value::String(text)
        );
    }

    None
}

fn eval_string(
    expr: &str,
    variables: &HashMap<String, Value>,
) -> Option<String> {

    let mut parts:
        Vec<String> = Vec::new();

    let mut current =
        String::new();

    let mut in_string =
        false;

    let mut chars =
        expr.chars().peekable();

    while let Some(ch)
        = chars.next()
    {
        if ch == '"' {

            in_string =
                !in_string;

            current.push(ch);

            continue;
        }

        if ch == '+'
            &&
           !in_string
        {
            let part =
                current.trim();

            if !part.is_empty() {

                parts.push(
                    part.to_string()
                );
            }

            current.clear();

            continue;
        }

        current.push(ch);
    }

    let part =
        current.trim();

    if !part.is_empty() {

        parts.push(
            part.to_string()
        );
    }

    let mut result =
        String::new();

    for part in parts {

        let part =
            part.trim();

        if part.starts_with('"')
            &&
           part.ends_with('"')
        {
            result.push_str(
                &part[1..part.len()-1]
            );
        }

        else if let Some(value)
            = variables.get(part)
        {
            match value {

                Value::String(s) => {
                    result.push_str(s);
                }

                Value::Number(n) => {
                    result.push_str(
                        &n.to_string()
                    );
                }

                Value::Bool(b) => {
                    result.push_str(
                        &b.to_string()
                    );
                }

                _ => {}
            }
        }

        else if let Ok(n)
            = part.parse::<f64>()
        {
            result.push_str(
                &n.to_string()
            );
        }

        else {

            return None;
        }
    }

    Some(result)
}

fn eval_math(
    expr: &str,
    variables: &HashMap<String, Value>,
) -> Option<f64> {

    if let Some((left, right))
        = expr.split_once('+')
    {
        return Some(
            get_number(
                left.trim(),
                variables
            )?
            +
            get_number(
                right.trim(),
                variables
            )?
        );
    }

    if let Some((left, right))
        = expr.split_once('-')
    {
        return Some(
            get_number(
                left.trim(),
                variables
            )?
            -
            get_number(
                right.trim(),
                variables
            )?
        );
    }

    if let Some((left, right))
        = expr.split_once('*')
    {
        return Some(
            get_number(
                left.trim(),
                variables
            )?
            *
            get_number(
                right.trim(),
                variables
            )?
        );
    }

    if let Some((left, right))
        = expr.split_once('/')
    {
        return Some(
            get_number(
                left.trim(),
                variables
            )?
            /
            get_number(
                right.trim(),
                variables
            )?
        );
    }

    get_number(
        expr.trim(),
        variables
    )
}

fn eval_condition(
    expr: &str,
    variables: &HashMap<String, Value>,
) -> Option<bool> {

    if let Some((left, right))
        = expr.split_once('>')
    {
        return Some(
            get_number(
                left.trim(),
                variables
            )?
            >
            get_number(
                right.trim(),
                variables
            )?
        );
    }

    if let Some((left, right))
        = expr.split_once('<')
    {
        return Some(
            get_number(
                left.trim(),
                variables
            )?
            <
            get_number(
                right.trim(),
                variables
            )?
        );
    }

    if let Some((left, right))
        = expr.split_once('=')
    {
        let left =
            eval_value(
                left.trim(),
                variables
            )?;

        let right =
            eval_value(
                right.trim(),
                variables
            )?;

        return Some(
            value_equal(
                &left,
                &right
            )
        );
    }

    None
}

fn value_equal(
    a: &Value,
    b: &Value,
) -> bool {

    match (a, b) {

        (
            Value::Number(x),
            Value::Number(y)
        ) => x == y,

        (
            Value::String(x),
            Value::String(y)
        ) => x == y,

        (
            Value::Bool(x),
            Value::Bool(y)
        ) => x == y,

        _ => false,
    }
}

fn get_number(
    text: &str,
    variables: &HashMap<String, Value>,
) -> Option<f64> {

    if let Ok(n)
        = text.parse::<f64>()
    {
        return Some(n);
    }

    if let Some(value)
        = variables.get(text)
    {
        match value {

            Value::Number(n) => {
                return Some(*n);
            }

            _ => {}
        }
    }

    None
}