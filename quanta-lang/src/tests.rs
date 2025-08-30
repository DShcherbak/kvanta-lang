use crate::{Compiler};
//use crate:utils::message::LinearCompilationMessage;

impl Compiler {
    // pub fn linear_compile_code(&mut self, source : &str) -> LinearCompilationMessage {
    //     self.linear_compile(source)
    // }
}

#[cfg(test)]
mod tests {
    
    use crate::Compiler;

    fn compile_ok(src: &str) -> Vec<String> {
        let mut compiler = Compiler::new();
        let msg = compiler.linear_compile(src);
        assert_eq!(msg.error_code, 0, "Unexpected compile error: {}", msg.get_error_message());
        let mut exec = msg.get_runtime();
        assert_eq!(Ok(()), exec.execute());
        let blocks = exec.get_commands();
        blocks.iter().fold(vec![], 
        |mut res, block| {
            let mut coms = block.get_commands();
            res.append(&mut coms);
            if block.sleep_for >= 0 {
                res.push(format!("sleep {}", block.sleep_for).into());
            }
            res
        })
    }

    #[test]
    fn test_file() {
        let file_path = "../grammar/test.txt";

        let contents = std::fs::read_to_string(file_path)
            .expect("Should have been able to read the file");
        assert!(contents.len() > 0);
        println!("{:?}\n=====================================", compile_ok(&contents));
    }

    #[test]
    fn circle_with_defaults() {
        let cmds = compile_ok("circle(320, 240, 100);");
        let expected = vec![
            "circle 320 240 100 fill=#ffffff stroke=#000000 width=1".to_string()
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn rectangle_with_custom_styles() {
        let src = r#"
            setFigureColor(Color::Red);
            setLineColor(Color::Green);
            setLineWidth(3);
            rectangle(10, 20, 110, 220);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // Red -> #e92331, Green -> #7eb786
            "rectangle 10 20 110 220 fill=#e92331 stroke=#7eb786 width=3".to_string()
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn style_persists_until_changed() {
        let src = r#"
            setFigureColor(Color::Blue);
            circle(0, 0, 5);
            rectangle(1, 2, 3, 4);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // Blue -> #2E73E6
            "circle 0 0 5 fill=#2e73e6 stroke=#000000 width=1".to_string(),
            "rectangle 1 2 3 4 fill=#2e73e6 stroke=#000000 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn style_updates_affect_following_commands_only() {
        let src = r#"
            setFigureColor(Color::Blue);
            circle(1, 2, 3);
            setFigureColor(Color::Yellow);
            rectangle(4, 5, 6, 7);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // Blue -> #2e73e6, Yellow -> #fde25d
            "circle 1 2 3 fill=#2e73e6 stroke=#000000 width=1".to_string(),
            "rectangle 4 5 6 7 fill=#fde25d stroke=#000000 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn line_command_is_geometry_only() {
        // Per spec: line(...) → "line x1 y1 x2 y2"
        let src = r#"
            setLineColor(Color::Green);
            setLineWidth(7);
            line(10, 10, 20, 20);
        "#;
        let cmds = compile_ok(src);
        let expected = vec!["line 10 10 20 20 stroke=#7eb786 width=7".to_string()];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn polygon_and_arc_match_shape_style_behavior() {
        let src = r#"
            setFigureColor(Color::Red);
            setLineColor(Color::Blue);
            setLineWidth(2);
            polygon(0,0, 10,0, 10,10, 0,10);
            arc(50, 60, 40, 0, 180);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // Red -> #e92331, Blue -> #2e73e6
            "polygon 0 0 10 0 10 10 0 10 fill=#e92331 stroke=#2e73e6 width=2".to_string(),
            "arc 50 60 40 0 180 fill=#e92331 stroke=#2e73e6 width=2".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn more_colors_map_correctly() {
        let src = r#"
            setFigureColor(Color::Pink);
            setLineColor(Color::Cyan);
            circle(5, 6, 7);
            setFigureColor(Color::White);
            setLineColor(Color::Black);
            rectangle(1,2,3,4);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // Pink -> #fb9ab5, Cyan -> #3ba8e7
            "circle 5 6 7 fill=#fb9ab5 stroke=#3ba8e7 width=1".to_string(),
            "rectangle 1 2 3 4 fill=#ffffff stroke=#000000 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

        #[test]
    fn circle_with_arithmetic_expressions() {
        let src = r#"
            circle(10+20, 30-5, 2*10);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            "circle 30 25 20 fill=#ffffff stroke=#000000 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn rectangle_with_division_and_modulo() {
        let src = r#"
            setFigureColor(Color::Cyan);
            rectangle(100/2, 10%3, 7*5, 80/4);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // 100/2=50, 10%3=1, 7*5=35, 80/4=20
            "rectangle 50 1 35 20 fill=#3ba8e7 stroke=#000000 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn mixed_operations_in_line_and_arc() {
        let src = r#"
            setLineColor(Color::Pink);
            line(5*2, 20/2, 50-25, 3+7);
            arc(100, 200, 10*2, 360/4, 50%7);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // line: (10, 10, 25, 10)
            "line 10 10 25 10 stroke=#fb9ab5 width=1".to_string(),
            // arc: (100, 200, 20, 90, 1)
            "arc 100 200 20 90 1 fill=#ffffff stroke=#fb9ab5 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

        #[test]
    fn precedence_without_parentheses() {
        let src = r#"
            circle(2+3*4, 20-6/2, 10%4+1);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // 2+3*4 = 14, 20-6/2 = 17, 10%4+1 = 3
            "circle 14 17 3 fill=#ffffff stroke=#000000 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn precedence_with_parentheses() {
        let src = r#"
            rectangle((2+3)*4, (20-6)/2, (10%4)+1, (8/2)*(3+1));
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // (2+3)*4 = 20, (20-6)/2 = 7, (10%4)+1 = 3, (8/2)*(3+1) = 16
            "rectangle 20 7 3 16 fill=#ffffff stroke=#000000 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn polygon_with_mixed_precedence_and_parentheses() {
        let src = r#"
            setFigureColor(Color::Yellow);
            polygon(1+2*3, (4+6)%5, 18/3-2, (2+2)*(3+1), 7, 8);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // 1+2*3 = 7, (4+6)%5 = 0, 18/3-2 = 4, (2+2)*(3+1) = 16
            "polygon 7 0 4 16 7 8 fill=#fde25d stroke=#000000 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn line_with_arithmetic_and_styles() {
        let src = r#"
            setLineColor(Color::Pink);
            setLineWidth(2*3);
            line(5*2, 20/2, 50-25, (3+7));
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // line: (10, 10, 25, 10), stroke=#fb9ab5, width=6
            "line 10 10 25 10 stroke=#fb9ab5 width=6".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn arc_with_arithmetic_and_line_style() {
        let src = r#"
            setLineColor(Color::Cyan);
            setLineWidth(4+1);
            arc(100/2, 200-50, 5*4, 360/4, 50%7);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // arc(50, 150, 20, 90, 1), stroke=#3ba8e7, width=5
            "arc 50 150 20 90 1 fill=#ffffff stroke=#3ba8e7 width=5".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

        #[test]
    fn int_variable_in_circle() {
        let src = r#"
            int x = 5;
            circle(x, x*2, x+10);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // x=5 → (5, 10, 15)
            "circle 5 10 15 fill=#ffffff stroke=#000000 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn color_variable_in_rectangle() {
        let src = r#"
            color b = Color::Red;
            setFigureColor(b);
            rectangle(0, 0, 20, 10);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            "rectangle 0 0 20 10 fill=#e92331 stroke=#000000 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    // #[test]
    // fn float_variable_in_arc() {
    //     let src = r#"
    //         float t = 4.5;
    //         arc(10, 20, t*2, 0, 180);
    //     "#;
    //     let cmds = compile_ok(src);
    //     let expected = vec![
    //         // t=4.5, t*2=9.0 (assuming compiler casts float→int)
    //         "arc 10 20 9 0 180 fill=#ffffff stroke=#000000 width=1".to_string(),
    //     ];
    //     assert_eq!(cmds, expected);
    // }

    // #[test]
    // fn bool_variable_in_line_width() {
    //     let src = r#"
    //         bool d = true;
    //         setLineWidth(d*5); // true→1, false→0 ?
    //         line(0, 0, 10, 10);
    //     "#;
    //     let cmds = compile_ok(src);
    //     let expected = vec![
    //         // d=true → 1*5=5
    //         "line 0 0 10 10 stroke=#000000 width=5".to_string(),
    //     ];
    //     assert_eq!(cmds, expected);
    // }

    #[test]
    fn mixed_variables_in_polygon() {
        let src = r#"
            int x = 2;
            color c = Color::Blue;
            setFigureColor(c);
            polygon(x, 3, x*10, 3*2, 3, 4);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // x=2, y=3.5 → polygon(2, 3, 20, 7)
            "polygon 2 3 20 6 3 4 fill=#2e73e6 stroke=#000000 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

        #[test]
    fn polygon_requires_at_least_six_args() {
        // Too few args (only 4): should be an error.
        let src = r#"
            polygon(0,0, 10,0);
        "#;
        let msg = Compiler::new().linear_compile_code(src);
        assert_ne!(msg.error_code, 0);
        assert!(msg.get_error_message().to_lowercase().contains("polygon"));
        assert!(msg.get_error_message().to_lowercase().contains("6"));
    }

    #[test]
    fn if_true_branch_executes() {
        let src = r#"
            if (3 > 2) {
                setFigureColor(Color::Green);
                circle(10, 20, 30);
            } else {
                setFigureColor(Color::Red);
                circle(0, 0, 5);
            }
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            "circle 10 20 30 fill=#7eb786 stroke=#000000 width=1".to_string(), // green
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn if_false_branch_executes_else() {
        let src = r#"
            if (1 == 2) {
                rectangle(0,0, 1,1);
            } else {
                setFigureColor(Color::Blue);
                rectangle(10, 10, 20, 20);
            }
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            "rectangle 10 10 20 20 fill=#2e73e6 stroke=#000000 width=1".to_string(), // blue
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn if_without_else_skips_when_false() {
        let src = r#"
            if (5 < 4) {
                circle(1,2,3);
            }
        "#;
        let cmds = compile_ok(src);
        assert!(cmds.is_empty(), "Expected no commands when condition is false and no else-block");
    }

    #[test]
    fn boolean_operator_precedence_and_parentheses() {
        // Expect true: !false && (true || false) == true
        let src = r#"
            if (!false && (true || false)) {
                setLineColor(Color::Cyan);
                setLineWidth(2);
                line(0, 0, 5*2, 4+6);
            } else {
                line(0,0,1,1);
            }
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            "line 0 0 10 10 stroke=#3ba8e7 width=2".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn comparisons_on_int_and_float_in_condition() {
        let src = r#"
            int x = 5;
            float t = 4.5;
            if ( ((x >= 5) && (t < 5.0)) && (x != 0) ) {
                setFigureColor(Color::Yellow);
                circle(x+5, x, x*2);
            } else {
                rectangle(0,0,1,1);
            }
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            // x+5 = 10, (int)4.5 -> 4, (int)(9.0) -> 9
            "circle 10 5 10 fill=#fde25d stroke=#000000 width=1".to_string(),
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn mixed_and_or_with_comparisons() {
        // Test && vs || precedence: && binds tighter than ||
        // Expr: (2 > 3) || (4 == 4 && 1 <= 2)  -> false || (true && true) -> true
        let src = r#"
            if ( 2 > 3 || 4 == 4 && 1 <= 2 ) {
                setFigureColor(Color::Pink);
                circle(1,1,1);
            } else {
                setFigureColor(Color::Red);
                circle(2,2,2);
            }
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            "circle 1 1 1 fill=#fb9ab5 stroke=#000000 width=1".to_string(), // pink
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn style_set_inside_if_persists_after_block() {
        // Assuming style changes are global (not block-scoped).
        let src = r#"
            if (1 == 1) {
                setFigureColor(Color::Red);
            }
            rectangle(0,0, 10,10);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            "rectangle 0 0 10 10 fill=#e92331 stroke=#000000 width=1".to_string(), // red persists
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn scope_set_inside_if_persists_after_block() {
        // Assuming style changes are global (not block-scoped).
        let src = r#"
            int x = 0;
            if (1 == 1) {
                x = 10;
                setFigureColor(Color::Red);
            }
            rectangle(0,0, x,x);
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            "rectangle 0 0 10 10 fill=#e92331 stroke=#000000 width=1".to_string(), // red persists
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn polygon_in_if_with_minimum_vertices_and_styles() {
        let src = r#"
            if ((3*3) == 9) {
                setFigureColor(Color::Cyan);
                setLineColor(Color::Black);
                setLineWidth(3);
                polygon(0,0, 10,0, 10,10);
            }
        "#;
        let cmds = compile_ok(src);
        let expected = vec![
            "polygon 0 0 10 0 10 10 fill=#3ba8e7 stroke=#000000 width=3".to_string(),
        ];
        assert_eq!(cmds, expected);
    }


}