using System.Threading.Tasks;
using Cheetah.Matches.Relay.Editor.NetworkCommandsViewer.Provider;
using Cheetah.Matches.Relay.Editor.UIElements.HistoryTextField;
using Cheetah.Matches.Relay.Editor.UIElements.StatusIndicator;
using Grpc.Core;

namespace Cheetah.Matches.Relay.Editor.NetworkCommandsViewer.UI.Controller
{
    public class SearchFieldController
    {
        private readonly HistoryTextField historyTextField;
        private readonly StatusIndicator statusIndicator;
        private readonly TracedCommandsProvider provider;

        public SearchFieldController(HistoryTextField historyTextField, StatusIndicator statusIndicator,
            TracedCommandsProvider provider)
        {
            this.historyTextField = historyTextField;
            this.statusIndicator = statusIndicator;
            this.provider = provider;
            this.historyTextField.RegisterOnChangeListener(ApplyFilter);
        }


        public async Task Update()
        {
            await historyTextField.Update();
        }


        private async Task ApplyFilter(string value)
        {
            try
            {
                statusIndicator.ResetStatus();
                await provider.SetFilter(value);
            }
            catch (RpcException e)
            {
                ProcessError(e);
            }
        }

        private void ProcessError(RpcException rpcException)
        {
            if (!rpcException.Status.Detail.Contains("Query")) return;

            var message = rpcException.Status.Detail.Replace("\\","").Replace("QueryError(","").Replace("\"","");
            message = message.Substring(0, message.Length - 1);
            statusIndicator.SetStatus("Syntax error in query expression. " + message, StatusIndicator.MessageType.Error);
        }

        public void Enabled(bool enabled)
        {
            historyTextField.SetEnabled(enabled);
        }
    }
}