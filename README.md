# barsh
Command line tool that translates questions into bash commands

## Demo

![](https://github.com/LucaSchinnerl/barsh/blob/7-add-demo/demos/hs_demo.gif)


## Installation:

### Step 1 Install Package
`cargo install --git https://github.com/LucaSchinnerl/barsh.git`

### Step 3 Set API-KEY
To run first set your OPENAI API key
- bash: `export GROQ_API_KEY=<Your GROQ-API key>`
- fish: `set -Ux GROQ_API_KEY <Your GROQ-API key>`

### Step 4 Run Barsh
Some examples:
- `barsh print hello world`
- `barsh find all files in this dir that end in .rs`
- `barsh what time is it`
