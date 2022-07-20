using System;
using Cheetah.Matches.Relay.Internal;
using Cheetah.Matches.Relay.Internal.Plugin.Routers.ByTemplate;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Income.ByTemplate
{
    public class DeletedFieldByTemplateIncomeCommands : IDisposable
    {
        private readonly ushort template;
        private readonly ReferenceList<DeletedField> stream = new();

        private readonly DeleteFieldRouterByTemplate router;
        private readonly CheetahClient client;

        public DeletedFieldByTemplateIncomeCommands(CheetahClient client, ushort template)
        {
            this.client = client;
            this.template = template;

            client.BeforeUpdateHook += BeforeUpdate;

            router = client.GetPlugin<DeleteFieldRouterByTemplate>();
            router.RegisterListener(template, OnDelete);
        }

        private void BeforeUpdate()
        {
            stream.Clear();
        }

        private void OnDelete(DeletedField deletedField)
        {
            ref var item = ref stream.Add();
            item = deletedField;
        }

        public ReadonlyReferenceList<DeletedField> GetStream()
        {
            return stream;
        }

        public void Dispose()
        {
            client.BeforeUpdateHook -= BeforeUpdate;
            router.UnRegisterListener(template, OnDelete);
        }
    }
}