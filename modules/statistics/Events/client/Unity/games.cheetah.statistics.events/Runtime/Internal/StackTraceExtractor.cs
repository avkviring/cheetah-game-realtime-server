using System;
using UnityEngine.Assertions;

namespace Cheetah.Statistics.Events.Internal
{
    public class StackTraceExtractor
    {
        private int lineNumber = -1;
        private string fileName;
        private string methodName;
        private readonly string error;

        public StackTraceExtractor(string stacktrace)
        {
            try
            {
                var lines = stacktrace.Split("\n");
                if (lines.Length <= 1) return;
                var line = lines[1];
                ParseMethodName(line);
                ParseLineNumber(line);
                ParseFileName(line);
            }
            catch (Exception e)
            {
                error = e.ToString();
            }
        }

        private void ParseMethodName(string line)
        {
            var index = line.IndexOf(")", StringComparison.Ordinal) + 1;
            methodName = line.Substring(0, index).Replace("at ","").Trim();
        }

        private void ParseFileName(string line)
        {
            var start = line.LastIndexOf("(", StringComparison.Ordinal);
            var end = line.LastIndexOf(".cs", StringComparison.Ordinal);
            if (start < 0 || end < 0)
            {
                return;
            }

            fileName = line.Substring(start + 1, end - start + 2).Replace("at ", "");
        }

        private void ParseLineNumber(string line)
        {
            var index = line.LastIndexOf("cs:", StringComparison.Ordinal);
            if (index <= 0) return;
            var value = line.Substring(index + 3, line.Length - index - 3).Replace(")", "");
            lineNumber = int.Parse(value);
        }

        public string GetFileName()
        {
            Assert.IsTrue(HasFileName());
            return fileName;
        }

        public string GetMethodName()
        {
            Assert.IsTrue(HasMethodName());
            return methodName;
        }

        public int GetLineNumber()
        {
            Assert.IsTrue(HasLineNumber());
            return lineNumber;
        }

        public string GetError()
        {
            return error;
        }

        public bool HasLineNumber()
        {
            return lineNumber > 0;
        }

        public bool HasFileName()
        {
            return fileName != null;
        }

        public bool HasMethodName()
        {
            return methodName != null;
        }
    }
}