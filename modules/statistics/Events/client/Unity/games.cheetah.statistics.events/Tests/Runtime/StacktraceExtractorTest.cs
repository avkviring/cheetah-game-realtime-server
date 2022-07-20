using Cheetah.Statistics.Events.Internal;
using NUnit.Framework;

namespace Cheetah.Matches.Statistics.Events.Test.Tests.Runtime
{
    public class StacktraceExtractorTest
    {
        [Test]
        public void ShouldParseModuleAndLineNumber()
        {
            var stacktrace = @"UnityEngine.Debug:LogError (object)
Tests.Statistics.SenderTest/<TestLog>d__2:MoveNext () (at Assets/Tests/Statistics/SenderTest.cs:40)
UnityEngine.SetupCoroutine:InvokeMoveNext (System.Collections.IEnumerator,intptr) (at /Users/bokken/buildslave/unity/build/Runtime/Export/Scripting/Coroutines.cs:17)
";
            var extractor = new StackTraceExtractor(stacktrace);
            Assert.AreEqual(40, extractor.GetLineNumber());
            Assert.AreEqual("Assets/Tests/Statistics/SenderTest.cs", extractor.GetFileName());
            Assert.AreEqual("Tests.Statistics.SenderTest/<TestLog>d__2:MoveNext ()", extractor.GetMethodName());
        }

        [Test]
        public void ShouldParseModuleAndLineNumberForReleaseException()
        {
            var stacktrace = @"SendTaskError { source: SendError { .. } } 
  at Cheetah.Matches.Relay.Internal.ResultChecker.Check (System.Byte code) [0x00000] in <00000000000000000000000000000000>:0 
  at SyncariotStateSend.OnUpdate () [0x00000] in <00000000000000000000000000000000>:0 
  at GameLoopComponent`1[T].Run () [0x00000] in <00000000000000000000000000000000>:0 
  at GameLoop.FixedUpdate () [0x00000] in <00000000000000000000000000000000>:0 
UnityEngine.Debug:LogError(Object, Object)
GameLoopComponent`1:Run()
GameLoop:FixedUpdate()";
            var extractor = new StackTraceExtractor(stacktrace);
            Assert.AreEqual(false, extractor.HasFileName());
            Assert.AreEqual(false, extractor.HasLineNumber());
            Assert.AreEqual("Cheetah.Matches.Relay.Internal.ResultChecker.Check (System.Byte code)", extractor.GetMethodName());
        }

        [Test]
        public void ShouldNotExceptionWhenWrongFormat()
        {
            var stacktrace = @"";
            var extractor = new StackTraceExtractor(stacktrace);
            Assert.IsFalse(extractor.HasFileName());
            Assert.IsFalse(extractor.HasLineNumber());
            Assert.IsFalse(extractor.HasMethodName());
        }
    }
}