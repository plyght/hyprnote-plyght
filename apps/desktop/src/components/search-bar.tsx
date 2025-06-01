import { useLingui } from "@lingui/react/macro";
import clsx from "clsx";
import { SearchIcon, XIcon } from "lucide-react";
import { useState } from "react";

import { useHyprSearch } from "@/contexts/search";
import Shortcut from "./shortcut";

export function SearchBar() {
  const {
    searchQuery,
    searchInputRef,
    focusSearch,
    clearSearch,
    setSearchQuery,
  } = useHyprSearch((s) => ({
    searchQuery: s.query,
    searchInputRef: s.searchInputRef,
    focusSearch: s.focusSearch,
    clearSearch: s.clearSearch,
    setSearchQuery: s.setQuery,
  }));
  const { t } = useLingui();
  const [isFocused, setIsFocused] = useState(false);

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setSearchQuery(value);
  };

  return (
    <div
      className={clsx([
        "w-60 flex items-center gap-2 h-[34px]",
        "text-muted-foreground hover:text-foreground",
        "border border-border rounded-md px-2 py-2 bg-transparent",
        "hover:bg-background",
        isFocused && "bg-background",
        "transition-colors duration-200",
      ])}
      onClick={() => focusSearch()}
    >
      <SearchIcon className="h-4 w-4 text-muted-foreground" />
      <input
        ref={searchInputRef}
        type="text"
        value={searchQuery}
        onChange={handleInputChange}
        onFocus={() => setIsFocused(true)}
        onBlur={() => setIsFocused(false)}
        placeholder={t`Search...`}
        className="flex-1 bg-transparent outline-none text-xs"
      />
      {searchQuery
        ? (
          <XIcon
            onClick={() => clearSearch()}
            className="h-4 w-4 text-muted-foreground hover:text-foreground"
          />
        )
        : <Shortcut macDisplay="⌘K" windowsDisplay="Ctrl+K" />}
    </div>
  );
}
