<?php

function App($props) {
  $node = <div class="card" />;
//          ^^^ @tag
//              ^^^^^ @attribute
//                    ^^^^^^ @string

  $child = <span>Hello</span>;
//           ^^^^ @tag
//                ^^^^^ @string
}
