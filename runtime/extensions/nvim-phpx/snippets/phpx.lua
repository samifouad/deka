local ls = require("luasnip")
local s = ls.snippet
local t = ls.text_node
local i = ls.insert_node
local fmt = require("luasnip.extras.fmt").fmt

return {
  s("pfn", fmt([[function {}({}: {}): {} {{
    {}
}}]], {
    i(1, "name"),
    i(2, "$arg"),
    i(3, "string"),
    i(4, "string"),
    i(5, "return ''"),
  })),

  s("pstruct", fmt([[struct {} {{
    ${}: {}
}}]], {
    i(1, "Name"),
    i(2, "field"),
    i(3, "string"),
  })),

  s("pimport", fmt([[import {{ {} }} from '{}']], {
    i(1, "symbol"),
    i(2, "module"),
  })),

  s("pjsx", fmt([[function {}($props: Object<{{ {}: string }}>) {{
    return <div>{{$props.{}}}</div>
}}]], {
    i(1, "Component"),
    i(2, "message"),
    i(3, "message"),
  })),

  s("pfm", fmt([[---
import {{ {} }} from '{}'

${} = {}
---
<{} />]], {
    i(1, "Component"),
    i(2, "module"),
    i(3, "data"),
    i(4, "null"),
    i(5, "Component"),
  })),
}
