# Syntax Analyzer

## Description
The following project comprises the first 2 stages of a compiler-compiler
- <b>Lexic Analyzer Generator:</b><br>
With use of the Yet Another Lexer (.yal) format, we can define different regexes and their related action for a lexic analysis of a raw input file. This stage uses different techniques such as Direct Generation of Deterministic Finite Automatons, tokenization of inputs, Hoppcroft's minimizng algorithm, amongst others to be able to be a general solution for correctly identifying regexes.
- <b>Syntax Analyzer Generator:</b><br>
Inspired by Yet Another Parser notation (.yalp) this second stage uses productions defined for a Context Free Grammar to be able to identify correct or incorrect syntax on tokenized input. This module makes direct use of the Lexic Analysis tokenized output to be able to realize its purpose. This step comprises two main implementations for syntax analysis: an SLR(1) compliant one and a LALR one, which can be chosen before running in the config file.

## Configuration:
The ```config.json``` file in the project houses all the configuration parameters for the project to run propperly. The following describes its structure and explains briefly the different fields that comprise it:
```
    // Parsing Method
    "parse_method": "SLR" | "LALR"

    // Debug Messages
    "debug": {
        "generation": True | False,
        "parsing": True | False
    },
    
    // Visualization
    // It can take the value of your path of where you want the visualization to go or null for not generating it
    "vis": {
        "slr_png": "destiniy/path/for/slr_image.png" | null,
        "parse_table": "destiniy/path/for/parsing_table.txt" | null,
        "parse_steps": "destiniy/path/for/parse_steps" | null,
        "symbol_table": "destiniy/path/for/table" | null,
        "grammar_tree": ".destiniy/path/for/grammar_tree" | null,
        "dfa": "destiniy/path/for/dfa_image.png and .dot" | null
    }
```

## How to Run:
1. Fill up your configuration file with the required information
2. Generate your parser by running:<br>
        ```
        cargo run --bin syntax_analyzer -- ./path/to/lex.yal ./path/to/syn.yalp
        ```
    This will fill the .ron (Rust Object Notation) with the information necessary for the actual parsing of an input. You can se it like we are Compiling the Compiler in this step.
3. Parse an input by running:<br>
        ```
        cargo run --bin parser -- ./path/to/input.txt
        ```
    Following the idea of the last step, here we've already generated or "compiler" (which for this project is just a syntax and lexic analizer) so now we can see if an input follows the syntax of a grammar and the rules of the regular expresions previously defined.

## Developers:
#### Diego Garcia<br>
- <a  href="https://github.com/DiegoGarV">DiegoGarV</a>
#### Irving Fabricio<br>
- <a  href="https://github.com/wwIrvingww">wwIrvingww</a>
#### Jose Marchena
- <a  href="https://github.com/MarchMol">MarchMol</a>