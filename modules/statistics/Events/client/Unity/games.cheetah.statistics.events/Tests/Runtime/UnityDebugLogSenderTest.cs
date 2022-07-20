using System;
using System.Threading;
using Cheetah.Statistics.Events;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;
using Assert = UnityEngine.Assertions.Assert;

namespace Cheetah.Matches.Statistics.Events.Test.Tests.Runtime
{
    public class UnityDebugLogSenderTest
    {
        StubSender stubSender;

        [SetUp]
        public void SetUp()
        {
            stubSender = new StubSender();
            new UnityDebugLogSender(new StatisticsSession(stubSender));
            stubSender.items.Clear();
        }

        [Test]
        public void ShouldNotSendLogAndWarning()
        {
            Debug.Log("hello from log");
            Debug.LogWarning("hello from warning");
            Assert.AreEqual(stubSender.items.Count, 0);
        }

        [Test]
        public void ShouldSendError()
        {
            LogAssert.ignoreFailingMessages = true;
            Debug.LogError("hello from error");
            Assert.AreEqual(stubSender.items.Count, 1);
        }

        [Test]
        public void ShouldSendException()
        {
            LogAssert.ignoreFailingMessages = true;
            Debug.LogException(new NullReferenceException());
            Assert.AreEqual(stubSender.items.Count, 1);

            var item = stubSender.items[0];
            Assert.AreEqual("Packages/games.cheetah.statistics.events/Tests/Runtime/UnityDebugLogSenderTest.cs", item.labels["file"]);
            Assert.IsNotNull(item.labels["line"]);
        }

        [Test]
        public void ShouldSpamProtectForException()
        {
            LogAssert.ignoreFailingMessages = true;
            for (var i = 0; i < UnityDebugLogSender.AllowedSpamCount + 5; i++)
            {
                Debug.LogException(new NullReferenceException());
            }

            Assert.AreEqual(stubSender.items.Count, UnityDebugLogSender.AllowedSpamCount - 1);

            var item = stubSender.items[UnityDebugLogSender.AllowedSpamCount - 5];
            Assert.IsNotNull(item.labels["count"]);
        }

        [Test]
        public void ShouldSpamProtectForReleaseException()
        {
            var error = @"
 SendTaskError { source: SendError { .. } } 
   at Cheetah.Matches.Relay.Internal.ResultChecker.Check (System.Byte code) [0x00000] in <00000000000000000000000000000000>:0 
  at SyncariotStateSend.OnUpdate () [0x00000] in <00000000000000000000000000000000>:0 
  at GameLoopComponent`1[T].Run () [0x00000] in <00000000000000000000000000000000>:0 
  at GameLoop.FixedUpdate () [0x00000] in <00000000000000000000000000000000>:0 
UnityEngine.Debug:LogError(Object, Object)
GameLoopComponent`1:Run()
GameLoop:FixedUpdate()
";
            
            LogAssert.ignoreFailingMessages = true;
            for (var i = 0; i < UnityDebugLogSender.AllowedSpamCount + 5; i++)
            {
                Debug.LogError(error);
            }

            Assert.AreEqual(stubSender.items.Count, UnityDebugLogSender.AllowedSpamCount - 1);

            var item = stubSender.items[UnityDebugLogSender.AllowedSpamCount - 5];
            Assert.IsNotNull(item.labels["count"]);
        }
        
        [Test]
        public void ShouldSpamProtectForError()
        {
            var error = "Some error";
            LogAssert.ignoreFailingMessages = true;
            for (var i = 0; i < UnityDebugLogSender.AllowedSpamCount + 5; i++)
            {
                Debug.LogError(error);
            }
            Assert.AreEqual(stubSender.items.Count, UnityDebugLogSender.AllowedSpamCount - 1);
            var item = stubSender.items[UnityDebugLogSender.AllowedSpamCount - 5];
            Assert.IsNotNull(item.labels["count"]);
        }
        [Test]
        public void ShouldSendSpamAfterSilencePeriod()
        {
            var error = "Some error";
            LogAssert.ignoreFailingMessages = true;
            UnityDebugLogSender.SilenceSpamPeriodInMs = 100;
            for (var i = 0; i < UnityDebugLogSender.AllowedSpamCount + 1; i++)
            {
                if (i == UnityDebugLogSender.AllowedSpamCount)
                {
                   Thread.Sleep(UnityDebugLogSender.SilenceSpamPeriodInMs+1);
                }
                Debug.LogError(error);
            }
            Assert.AreEqual(stubSender.items.Count, UnityDebugLogSender.AllowedSpamCount);
            var item = stubSender.items[UnityDebugLogSender.AllowedSpamCount - 1];
            Assert.AreEqual(item.labels["count"], (UnityDebugLogSender.AllowedSpamCount + 1).ToString());
        }
        
    }
}