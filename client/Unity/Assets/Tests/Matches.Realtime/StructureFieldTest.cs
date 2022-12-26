using System.Linq;
using System.Threading;
using NUnit.Framework;
using Shared;
using Shared.Types;
using Tests.Matches.Realtime.Helpers;

namespace Tests.Matches.Realtime
{
    public class StructureFieldTest : AbstractTest
    {
        [Test]
        public void ShouldSet()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // изменяем структуру
            var turretsParams = new TurretsParamsStructure()
            {
                Damage = 1.5,
                Speed = 154
            };
            clientA.Writer.SetStructure(in createdObject.ObjectId, TurretsParamsFieldId, in turretsParams);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var changes = clientB.Reader.GetModifiedStructures<TurretsParamsStructure>(777, TurretsParamsFieldId);
            var actual = changes[createdObject.ObjectId];
            Assert.AreEqual(turretsParams.Damage, actual.Damage);
            Assert.AreEqual(turretsParams.Speed, actual.Speed);
        }
    }
}