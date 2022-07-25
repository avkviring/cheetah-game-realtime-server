using System;
using System.Text;

namespace Cheetah.Matches.Realtime.Editor.Generator
{
    public static class Utils
    {
        internal static string AddTabs(string body)
        {
            var result = new StringBuilder();
            foreach (var line in body.Split("\n"))
            {
                result.AppendLine("\t" + line.TrimEnd());
            }

            return result.ToString();
        }

        public static string GetFullName(Type type)
        {
            return type.FullName.Replace("+", ".");
        }
    }
}