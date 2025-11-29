from mcp.server.fastmcp import FastMCP
from duckduckgo_search import DDGS
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize FastMCP server
mcp = FastMCP("web-search")

@mcp.tool()
def search(query: str, max_results: int = 3) -> str:
    """
    Perform a web search using DuckDuckGo.
    
    Args:
        query: The search query.
        max_results: Maximum number of results to return (default: 3).
    """
    logger.info(f"Searching for: {query}")
    try:
        with DDGS() as ddgs:
            results = list(ddgs.text(query, max_results=max_results))
            
        if not results:
            return "No results found."
            
        formatted = []
        for r in results:
            formatted.append(f"Title: {r['title']}\nLink: {r['href']}\nSnippet: {r['body']}")
            
        return "\n\n---\n\n".join(formatted)
    except Exception as e:
        logger.error(f"Search failed: {e}")
        return f"Search failed: {str(e)}"

if __name__ == "__main__":
    mcp.run()
