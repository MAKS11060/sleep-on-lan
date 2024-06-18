#!/usr/bin/env -S deno run -A

export const suspend = (
  hibernate: boolean = false,
  force: boolean = false,
  disableWakeEvent: boolean = false
) => {
  if (Deno.build.os == 'windows') {
    const {symbols} = Deno.dlopen('powrprof.dll', {
      // ref https://learn.microsoft.com/en-us/windows/win32/api/powrprof/nf-powrprof-setsuspendstate#syntax
      SetSuspendState: {
        parameters: ['bool', 'bool', 'bool'],
        result: 'bool',
      },
    })

    console.log(
      'result',
      symbols.SetSuspendState(hibernate, force, disableWakeEvent)
    )

  }
}

if (import.meta.main) {
  if (confirm('Go to sleep mode?')) {
    suspend(false, false, false)
  }
}
