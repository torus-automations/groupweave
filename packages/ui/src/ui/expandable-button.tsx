"use client";

import * as React from "react";
import { Button } from "./button";

const ExpandableButton = () => {
  const [isExpanded, setIsExpanded] = React.useState(false);

  return (
    <div className="fixed bottom-4 right-4 z-50">
      {isExpanded ? (
        <div className="flex flex-col items-end">
          <div className="bg-background border rounded-lg p-4 w-64 h-80 mb-2 shadow-lg">
            <p>This is the expanded content.</p>
          </div>
          <Button onClick={() => setIsExpanded(false)}>Close</Button>
        </div>
      ) : (
        <Button onClick={() => setIsExpanded(true)}>Open</Button>
      )}
    </div>
  );
};

export { ExpandableButton };
