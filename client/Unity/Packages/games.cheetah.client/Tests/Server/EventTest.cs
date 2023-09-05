using System.Collections.Generic;
using System.Linq;
using System.Threading;
using Games.Cheetah.Client.Tests.Server.Helpers;
using Games.Cheetah.Client.Tests.Server.Types;
using Games.Cheetah.Client.Types.Object;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Server
{
    public class EventTest : AbstractTest
    {
        private NetworkObject networkObject;
        private DropMineEvent dropMineEvent;

        [SetUp]
        public void SetUp()
        {
            networkObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            dropMineEvent = new DropMineEvent
            {
                MineId = 150
            };
        }
        
        [Test]
        public void TestEventIncomeCommands()
        {
            clientA.Writer.SendEvent(in networkObject.ObjectId, DropMineEventFieldIdId, in dropMineEvent);
            Thread.Sleep(200);
            clientB.Update();
            var eventsStream = clientB.Reader.GetEvents<DropMineEvent>(777, DropMineEventFieldIdId);
            var firstEvent = eventsStream.SearchFirst(it=>it.Item1==networkObject.ObjectId).Item2;
            Assert.AreEqual(dropMineEvent.MineId, firstEvent.MineId);
            eventsStream.Dispose();
        }

        [Test]
        public void TestTargetEventIncomeCommands()
        {
            clientA.Writer.SendEvent(in networkObject.ObjectId, DropMineEventFieldIdId, memberB.UserId, in dropMineEvent);
            Thread.Sleep(200);
            clientB.Update();
            var eventsStream = clientB.Reader.GetEvents<DropMineEvent>(777, DropMineEventFieldIdId);
            var firstEvent = eventsStream.SearchFirst(it=>it.Item1==networkObject.ObjectId).Item2;
            Assert.AreEqual(dropMineEvent.MineId, firstEvent.MineId);
            eventsStream.Dispose();
        }
        
        [Test]
        public void TestEventIncomeCommandsWithCollect()
        {
            clientA.Writer.SendEvent(in networkObject.ObjectId, DropMineEventFieldIdId, in dropMineEvent);
            Thread.Sleep(200);
            clientB.Update();
            var eventsStream = new List<(NetworkObjectId, DropMineEvent)>();
            clientB.Reader.CollectEvents<DropMineEvent>(777, DropMineEventFieldIdId, eventsStream);
            var firstEvent = eventsStream.First(it=>it.Item1==networkObject.ObjectId).Item2;
            Assert.AreEqual(dropMineEvent.MineId, firstEvent.MineId);
        }
    }
}