using System.Collections.Generic;
using System.Linq;
using System.Threading;
using Games.Cheetah.Client.Tests.Server.Helpers;
using Games.Cheetah.Client.Tests.Server.Types;
using Games.Cheetah.Client.Types.Object;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Server
{
    public class StructureFieldTest : AbstractTest
    {
        private TurretsParamsStructure turretsParams;
        private NetworkObject networkObject;

        [SetUp]
        public void SetUp()
        {
           
            networkObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            turretsParams = new TurretsParamsStructure()
            {
                Damage = 1.5,
                Speed = 154
            };
            clientA.Writer.SetStructure(in networkObject.ObjectId, TurretsParamsFieldId, in turretsParams);
            Thread.Sleep(200);            
            clientB.Update();
        }

        [Test]
        public void ShouldStructureWithNativeList()
        {
            var changes = clientB.Reader.GetModifiedStructures<TurretsParamsStructure>(777, TurretsParamsFieldId);
            var actual = changes.SearchFirst(it => it.Item1 == networkObject.ObjectId).Item2;
            Assert.AreEqual(turretsParams.Damage, actual.Damage);
            Assert.AreEqual(turretsParams.Speed, actual.Speed);
            changes.Dispose();
        }
        
        [Test]
        public void ShouldStructureWithList()
        {
            var changes = new List<(NetworkObjectId, TurretsParamsStructure)>();
            clientB.Reader.CollectModifiedStructures<TurretsParamsStructure>(777, TurretsParamsFieldId, changes);
            var actual = changes.First(it => it.Item1 == networkObject.ObjectId).Item2;
            Assert.AreEqual(turretsParams.Damage, actual.Damage);
            Assert.AreEqual(turretsParams.Speed, actual.Speed);
        }
    }
}