using System.Collections;
using System.Runtime.CompilerServices;
using System.Threading;
using AOT;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class DoubleValueTest : AbstractCommandTest
    {
        private double changedValue;
        private RelayObjectId changedObjectId;
        private ushort changedField;

        [Test]
        public void Test()
        {
            ClientCommands.SetCurrentClient(clientB);
            DoubleValueCommands.SetListener(Listener);

            ClientCommands.SetCurrentClient(clientA);
            DoubleValueCommands.Set(ref objectId, 1, 500.500);
            DoubleValueCommands.Increment(ref objectId, 1, 100.100);
            DoubleValueCommands.Increment(ref objectId, 1, 200.200);
            Thread.Sleep(200);

            ClientCommands.SetCurrentClient(clientB);
            ClientCommands.Receive();
            Assert.AreEqual(changedValue, 800.800, 0.1);
            Assert.AreEqual(changedField, 1);
            Assert.AreEqual(changedObjectId, objectId);
        }


        [MonoPInvokeCallback(typeof(DoubleValueCommands.Listener))]
        private void Listener(ref CommandMeta meta, ref RelayObjectId objectId, ushort fieldId, double value)
        {
            changedValue = value;
            changedObjectId = objectId;
            changedField = fieldId;
        }
    }
}