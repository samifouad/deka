<?php
// https://www.php.net/manual/en/functions.user-defined.php
function describe(string $item, bool $detail = false): string {
    $message = "Item: $item";
    if ($detail) {
        $message .= " (detailed information)";
    }
    return $message;
}

echo describe("php-rs") . "\n";
echo describe("PHP", true) . "\n";
