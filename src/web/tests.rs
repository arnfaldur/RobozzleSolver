use super::*;

#[test]
fn parse_json() {
    let level_json: Puzzle = puzzle_from_string(LEVEL_JSON);
    let level_json: Puzzle = puzzle_from_string(LEVEL_JSON2);
    assert!(true);
}

#[test]
fn test_get_local_puzzle() {
    get_local_level(100);
}

const LEVEL_JSON: &str = "{
\"About\": \"Collect starfruit! (See comments for hints - coming soon)\",
\"AllowedCommands\": \"0\",
\"Colors\": [
\"RRRRRRRRRRRRRRRR\",
\"RRRRRRGGGGRRRRRR\",
\"RRRRRGGGGGGRRRRR\",
\"RRRRRGGGGGGRRRRR\",
\"RRRRRGGRRGGGGGRR\",
\"RRRRRRGRRGGGGGGR\",
\"RRGGGRRRRRGRGGGR\",
\"RGGRRRRRRRRRRGGR\",
\"RGGGGGGRRRGGGGGR\",
\"RGGGGGRRRRRGGGRR\",
\"RRGGGRRRRRRRRRRR\",
\"BBBBBBBRRBBBBBBB\"
],
\"CommentCount\": \"0\",
\"DifficultyVoteCount\": \"0\",
\"DifficultyVoteSum\": \"0\",
\"Disliked\": \"0\",
\"Featured\": \"false\",
\"Id\": \"1874\",
\"Items\": [
\"################\",
\"######****######\",
\"#####*....*#####\",
\"#####*....*#####\",
\"#####*....****##\",
\"######*...*..**#\",
\"##***##..#..*.*#\",
\"#*............*#\",
\"#*....*..#*...*#\",
\"#*...*#..##***##\",
\"##***##..#######\",
\"...****.........\"
],
\"Liked\": \"0\",
\"RobotCol\": \"8\",
\"RobotDir\": \"0\",
\"RobotRow\": \"3\",
\"Solutions\": \"1\",
\"SubLengths\": [
\"10\",
\"6\",
\"4\",
\"2\",
\"0\"
],
\"SubmittedBy\": \"masterluk\",
\"SubmittedDate\": \"2010-04-10T12:56:13.157\",
\"Title\": \"Tree of Balance\"
}";

const LEVEL_JSON2: &str = "
{
  \"About\": {},
  \"AllowedCommands\": \"0\",
  \"Colors\": [
    \"BRGBRBBBBBGBBBGB\",
    \"BBBBBBGBBBBBGBBB\",
    \"BGBBGRBBBGBBBBBB\",
    \"GBRBBBBBRBBBBGBB\",
    \"BGBGBRBGBBRBBRBB\",
    \"RRBBRBBBBBBBGBBG\",
    \"GBBBBBBGBBBRBBBB\",
    \"BGBRBRBGBGBGBGBB\",
    \"BBBBBBGBBBBBBBGB\",
    \"RBBRBBRBBBGBBBRB\",
    \"BBBBRBBBRBRBBBBB\",
    \"BBBBBBBGBBBBRBBR\"
  ],
  \"CommentCount\": \"0\",
  \"DifficultyVoteCount\": \"43\",
  \"DifficultyVoteSum\": \"158\",
  \"Disliked\": \"7\",
  \"Featured\": \"false\",
  \"Id\": \"81\",
  \"Items\": [
    \"...........***..\",
    \".........*......\",
    \"..........*.....\",
    \".....*......*...\",
    \"......*.......*.\",
    \"........**......\",
    \"..*.....*.......\",
    \"....*...........\",
    \"..........*..*..\",
    \"................\",
    \"......*.........\",
    \"................\"
  ],
  \"Liked\": \"26\",
  \"RobotCol\": \"1\",
  \"RobotDir\": \"3\",
  \"RobotRow\": \"11\",
  \"Solutions\": \"128\",
  \"SubLengths\": [\"4\", \"4\", \"0\", \"0\", \"0\"],
  \"SubmittedBy\": \"seka\",
  \"SubmittedDate\": \"2009-03-01T16:01:20.233\",
  \"Title\": \"Find a way\"
}
";
