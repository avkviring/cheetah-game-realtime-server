using System;
using System.Collections.Generic;
using Cheetah.Matches.Relay.Internal.Plugin;
using Cheetah.Matches.Relay.Types.Object;

namespace Cheetah.Matches.Relay.Income.ByTemplate
{
    /// <summary>
    /// Поток событий создания игровых объектов с определенным типом шаблона, сбрасывается каждый кадр
    /// </summary>
    public class CreatedObjectByTemplateIncomeCommands : IDisposable
    {
        private readonly List<CheetahObjectConstructor> constructors = new List<CheetahObjectConstructor>();
        private readonly CheetahClient client;
        private readonly ushort template;

        public CreatedObjectByTemplateIncomeCommands(CheetahClient client, ushort template)
        {
            this.template = template;
            this.client = client;
            this.client.BeforeUpdateHook += BeforeUpdate;
            var collector = client.GetPlugin<ObjectConstructorCollector>();
            collector.RegisterListener(template, OnCreated);
        }


        private void OnCreated(CheetahObjectConstructor constructor)
        {
            constructors.Add(constructor);
        }

        private void BeforeUpdate()
        {
            constructors.Clear();
        }


        public IReadOnlyList<CheetahObjectConstructor> GetStream()
        {
            return constructors;
        }


        public void Dispose()
        {
            client.BeforeUpdateHook -= BeforeUpdate;
            var collector = client.GetPlugin<ObjectConstructorCollector>();
            collector.UnRegisterListener(template, OnCreated);
        }
    }
}