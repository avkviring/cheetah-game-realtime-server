using System;

internal static class Utils
{
    internal static void CheckResult(bool result)
    {
        if (!result)
        {
            throw new Exception($"Embedded server by id not found");
        }
    }
}