# Set UTF-8 encoding for output to avoid garbled text
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

# Detect language
$culture = [System.Globalization.CultureInfo]::CurrentUICulture.Name
$isChinese = $culture -like "zh-*"

# Define messages
if ($isChinese)
{
    $msg_existing_tags = "`n喵喵，这是项目里现有的标签哦："
    $msg_no_tags = "  （未找到任何标签）"
    $msg_input_name = "`n请输入要删除的标签名（多个标签请用空格隔开）："
    $msg_empty_name = "标签名不能为空！"
    $msg_deleting_local = "--> 正在删除本地标签"
    $msg_deleting_remote = "--> 正在删除远程标签"
    $msg_done = "`n完成啦！"
}
else
{
    $msg_existing_tags = "`nHere are the existing tags in your project:"
    $msg_no_tags = "  (no tags found)"
    $msg_input_name = "`nenter the tag names to delete (separate multiple tags with spaces):"
    $msg_empty_name = "tag name cannot be empty!"
    $msg_deleting_local = "--> Deleting local tag"
    $msg_deleting_remote = "--> Deleting remote tag"
    $msg_done = "`nDone!"
}

# Show existing tags
Write-Host $msg_existing_tags -ForegroundColor Cyan
$all_tags = git tag
if ($all_tags)
{
    foreach ($tag in $all_tags)
    {
        Write-Host "  $tag" -ForegroundColor Yellow
    }
}
else
{
    Write-Host $msg_no_tags -ForegroundColor DarkGray
}

# Ask for tag names
$tags_to_delete_string = (Read-Host $msg_input_name).Trim()

# Check empty input
if ([string]::IsNullOrWhiteSpace($tags_to_delete_string))
{
    Write-Host $msg_empty_name -ForegroundColor Red
    exit 1
}

$remote_name = "origin"

# Split the string into an array of tags
$tags_to_delete = $tags_to_delete_string.Split(' ', [System.StringSplitOptions]::RemoveEmptyEntries)

# Loop through each tag and delete it
foreach ($tag in $tags_to_delete)
{
    Write-Host "$msg_deleting_local '$tag' ..." -ForegroundColor Yellow
    git tag -d $tag | Out-Null

    Write-Host "$msg_deleting_remote '$tag' ..." -ForegroundColor Yellow
    git push $remote_name :refs/tags/$tag | Out-Null
}

Write-Host $msg_done -ForegroundColor Cyan
