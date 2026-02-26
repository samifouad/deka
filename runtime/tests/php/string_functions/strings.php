<?php
// https://www.php.net/manual/en/ref.strings.php
$text = "PHP and php-rs";
echo "Original: $text\n";
echo "Substring: " . substr($text, 0, 3) . "\n";
echo "Replace: " . str_replace("php-rs", "GPT", $text) . "\n";
