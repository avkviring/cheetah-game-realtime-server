using System.Collections;
using System.Runtime.CompilerServices;
using System.Threading;
using AOT;
using NUnit.Framework;
using NUnit.Framework.Constraints;
using UnityEngine;
using UnityEngine.TestTools;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class DoubleValueTest : AbstractTest
    {
        private double changedValue;
        private CheetahObjectId changedObjectId;
        private ushort changedField;

        [Test]
        public void Test()
        {
            CheetahClient.SetCurrentClient(ClientB);
            CheetahDouble.SetListener(Listener);

            CheetahClient.SetCurrentClient(ClientA);
            CheetahDouble.Set(ref ObjectId, 1, 500.500);
            CheetahDouble.Increment(ref ObjectId, 1, 100.100);
            CheetahDouble.Increment(ref ObjectId, 1, 200.200);
            Thread.Sleep(200);

            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.Receive();
            Assert.AreEqual(changedValue, 800.800, 0.1);
            Assert.AreEqual(changedField, 1);
            Assert.AreEqual(changedObjectId, ObjectId);
        }


        [MonoPInvokeCallback(typeof(CheetahDouble.Listener))]
        private void Listener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort fieldId, double value)
        {
            changedValue = value;
            changedObjectId = objectId;
            changedField = fieldId;
        }
    }
}