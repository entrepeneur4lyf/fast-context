import asyncio
from fast_context import FastContextAnalyzer

async def main():
    a = FastContextAnalyzer(project_root=".")
    await a.analyze_async()
    funcs = await a.find_symbols_by_kind_async("function")
    print("functions:", len(funcs))

    await a.start_watching()
    # simulate: after editing files, call analyze again or rely on caches being marked dirty
    await a.analyze_async()

if __name__ == "__main__":
    asyncio.run(main())

