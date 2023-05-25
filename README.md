# barsh
Command line tool that translates questions into bash commands

## Installation:

### Step 1 Install Package
`cargo install --git git@github.com:LucaSchinnerl/barsh.git`

### Step 3 Set API-KEY
To run first set your OPENAI API key
- bash: `export OPENAI_SK=<Your OEPNAI-API key>`
- fish: `set -Ux OPENAI_SK <Your OEPNAI-API key>`

### Step 4 Run Barsh
Some examples:
- `barsh print hello world`
- `barsh find all files in this dir that end in .rs`
- `barsh what time is it`
