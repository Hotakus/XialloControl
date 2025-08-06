# Set UTF-8 encoding for output to avoid garbled text
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

# Detect language
$culture = [System.Globalization.CultureInfo]::CurrentUICulture.Name
$isChinese = $culture -like "zh-*"

# Define messages
if ($isChinese)
{
    $msg_existing_tags = "`n当前已有的标签："
    $msg_no_tags = "  （未找到标签）"
    $msg_input_name = "`n请输入标签名（小写）"
    $msg_empty_name = "标签名不能为空！"
    $msg_input_msg = "请输入标签说明"
    $msg_local_deleted = "正在删除本地标签"
    $msg_local_not_found = "本地标签未找到。"
    $msg_remote_deleted = "正在删除远程标签"
    $msg_creating = "正在创建标签"
    $msg_pushing = "正在推送标签到远程"
    $msg_done = "`n完成喵～"
}
else
{
    $msg_existing_tags = "`nexisting tags:"
    $msg_no_tags = "  (no tags found)"
    $msg_input_name = "`nenter tag name (lowercase)"
    $msg_empty_name = "tag name cannot be empty!"
    $msg_input_msg = "enter tag message"
    $msg_local_deleted = "deleting local tag"
    $msg_local_not_found = "local tag not found."
    $msg_remote_deleted = "deleting remote tag"
    $msg_creating = "creating tag"
    $msg_pushing = "pushing tag to remote"
    $msg_done = "`ndone!"
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

# Ask for tag name
$tag_name = Read-Host $msg_input_name

# Check empty input
if ($null -eq $tag_name -or $tag_name.Trim() -eq "")
{
    Write-Host $msg_empty_name -ForegroundColor Red
    exit 1
}

# Ask for tag message
$tag_msg = Read-Host $msg_input_msg
$remote_name = "origin"

# Delete local tag if exists
if (git tag | Select-String -Pattern "^$tag_name$")
{
    Write-Host "$msg_local_deleted $tag_name ..." -ForegroundColor Yellow
    git tag -d $tag_name | Out-Null
}
else
{
    Write-Host $msg_local_not_found -ForegroundColor DarkGray
}

# Delete remote tag
Write-Host "$msg_remote_deleted $tag_name ..." -ForegroundColor Yellow
git push $remote_name ":refs/tags/$tag_name" | Out-Null

# Create tag
Write-Host "$msg_creating $tag_name ..." -ForegroundColor Green
git tag -a $tag_name -m $tag_msg

# Push to remote
Write-Host "$msg_pushing $tag_name ..." -ForegroundColor Green
git push $remote_name $tag_name

Write-Host $msg_done -ForegroundColor Cyan
