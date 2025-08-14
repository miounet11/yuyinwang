package com.drop.tool.loc;

import static com.blankj.utilcode.util.ThreadUtils.runOnUiThread;
import android.util.Log;
import androidx.annotation.NonNull;
import com.blankj.utilcode.util.LogUtils;
import com.drop.tool.ali.AliRecogStatus;
import com.drop.tool.ali.AliRecognizerHelperListener;
import com.drop.tool.ali.AliSpeechTranscriberCallbackHandler;
import com.drop.tool.ui.act.recorderToText.SSLSocketClient;
import com.drop.tool.utils.AppExecutors;
import com.drop.tool.utils.AudioUtil;
import com.mp.common.BuildConfig;
import com.mp.common.audio.AudioRecordHelper;
import com.mp.common.audio.AudioRecordListener;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;
import java.util.List;
import okhttp3.OkHttpClient;
import okhttp3.Request;
import okhttp3.Response;
import okhttp3.WebSocket;
import okhttp3.WebSocketListener;
import okio.ByteString;

/**
 * 阿里语音识别帮助类。
 * 另起任务采集语音数据并发送数据给阿里sdk识别，识别回调接口后发送handler message统一处理识别结果
 * <p>
 * 这个类主要和AliRecognizerHelper类实现的功能一样，主要识别数据从文件方式来源
 * <p>
 * 不使用sdk内置录音识别，自定义录音机类，实现录音数据调用阿里识别
 */
public class LocRecognizerManager implements AudioRecordListener {
    private static String TAG = "AliRecordHelper";
//    public static final String DEFAULT_HOST = "http://bdchat.ijxsb.cn";
    public static final String DEFAULT_HOST = "http://luyin.jiqizu.cn";
    //    public static final String DEFAULT_HOST = "ws://192.168.2.41:10095";
    public static final String MODE = "2pass";
    public static final String CHUNK_SIZE = "5, 10, 5";
    public static final int CHUNK_INTERVAL = 10;
    private String hotWords = "阿里巴巴 20\n达摩院 20\n夜雨飘零 20\n";
    private String asrText = "";


    //对语音识别回调接口的统一处理
    private AliSpeechTranscriberCallbackHandler handler;
    //采集音频数据任务 使用内置录音回调接口就需要使用启动该任务
    //private RecordTask recordTask;
    //识别开始的时间
    private long recogStartTime;
    //记录的时长
    private long recordedDuration;
    public static long time;

    private AudioRecordHelper audioRecord;

    //计时器
    private Runnable timeRunnable = new Runnable() {
        @Override
        public void run() {
            if (recogStartTime > 0) {
                time = recordedDuration + (System.currentTimeMillis() - recogStartTime);
                handler.getAliRecordHelperListener().recogTime(time);
                handler.postDelayed(this, 500);
            }
        }
    };

    public LocRecognizerManager(AliRecognizerHelperListener listener) {
        //UI在主线程更新
        this.handler = new AliSpeechTranscriberCallbackHandler(listener);
    }

    /**
     * 初始化一些实例
     */
    public void initRecog() {
        if (audioRecord == null) {
            audioRecord = new AudioRecordHelper(this);
            audioRecord.setRecordPcmFilePath(AudioUtil.getRecordPcmPath());
        }
        //这里要切回主线程
        AppExecutors.getInstance().mainThread().execute(() ->
                sendecogStatus(AliRecogStatus.RECOG_READY)
        );
    }

    WebSocket webSocket;

    /**
     * 开始识别
     */
    public void startRecog() {
        OkHttpClient client = new OkHttpClient.Builder()
                .sslSocketFactory(SSLSocketClient.getSSLSocketFactory(), SSLSocketClient.getX509TrustManager())
                .hostnameVerifier(SSLSocketClient.getHostnameVerifier())
                .build();

        Request request = new Request.Builder()
                .url(DEFAULT_HOST)
                .build();

        webSocket = client.newWebSocket(request, new WebSocketListener() {

            @Override
            public void onOpen(@NonNull WebSocket webSocket, @NonNull Response response) {
                Log.d(TAG, "WebSocket连接成功");
//                runOnUiThread(() -> Toast.makeText(getApplicationContext(), "WebSocket连接成功", Toast.LENGTH_SHORT).show());
            }

            @Override
            public void onMessage(@NonNull WebSocket webSocket, @NonNull String text) {
                Log.d(TAG, "s: " + text);

                try {
                    JSONObject jsonObject = new JSONObject(text);
                    String t = jsonObject.getString("text");
                    boolean isFinal = jsonObject.getBoolean("is_final");
                    if (!t.isEmpty()) {
                        String mode = jsonObject.getString("mode");
                        if (mode.equals("2pass-offline")) {
                            String textStr = jsonObject.getString("text");
                            JSONArray stampSents = jsonObject.getJSONArray("stamp_sents");

                            JSONObject stampSentsObject = (JSONObject) stampSents.get(stampSents.length() - 1);
                            long start = stampSentsObject.getLong("end");
                            asrText = "";
                            runOnUiThread(() ->
                                    handler.getAliRecordHelperListener().recogFinalResult(textStr, start)
                            );
                        } else {
                            String textSeg = jsonObject.getString("text");
                            asrText = asrText + textSeg;
                            runOnUiThread(() ->
                                    handler.getAliRecordHelperListener().recogTempResult(asrText)
                            );
                        }
                    }
//                    if (!(allAsrText + asrText).isEmpty()) {
//                        Log.d(TAG, "dddddddOnMessage: " + allAsrText + asrText);
//                        runOnUiThread(() ->
//                                handler.getAliRecordHelperListener().recogFinalResult(allAsrText + asrText, 0)
//                        );
//                    }
                    if (isFinal) {
                        webSocket.close(1000, "关闭WebSocket连接");
                    }
                } catch (JSONException e) {
                    throw new RuntimeException(e);
                }
            }

            @Override
            public void onClosing(@NonNull WebSocket webSocket, int code, @NonNull String reason) {
                Log.d(TAG, "WebSocket关闭连接: " + reason);
            }

            @Override
            public void onFailure(@NonNull WebSocket webSocket, @NonNull Throwable t, Response response) {
                Log.d(TAG, "WebSocket连接失败: " + t + ": " + response);
//                runOnUiThread(() -> Toast.makeText(getApplicationContext(), "WebSocket连接失败：" + t, Toast.LENGTH_SHORT).show());
            }
        });
        String message = getMessage(true);
        Log.d(TAG, "WebSocket发送消息: " + message);
        webSocket.send(message);
        startTime();
        sendecogStatus(AliRecogStatus.RECOG_START);
        if (audioRecord != null)
            audioRecord.startRecord();

    }

    public void pauseRecorg() {
        handler.stopRecog();
        // 第八步，停止本次识别
        this.stopTime();

        if (audioRecord != null)
            audioRecord.pauseRecord();

        this.sendecogStatus(AliRecogStatus.RECOG_PAUSE);
    }

    public void stopRecog() {
        handler.stopRecog();
        stopTime();
        sendecogStatus(AliRecogStatus.RECOG_STOP);

        if (audioRecord != null)
            audioRecord.stopRecord();
    }

    public void canelRecog() {
        sendecogStatus(AliRecogStatus.RECOG_CANEL);
    }

    /**
     * 退出页面时候
     */
    public void release() {
        if (audioRecord != null)
            audioRecord.release();
    }

    /**
     * 开始时间
     */
    private void startTime() {
        recogStartTime = System.currentTimeMillis();
        handler.post(timeRunnable);
    }

    /**
     * 停止时间
     */
    private void stopTime() {
        if (recogStartTime > 0)
            recordedDuration = recordedDuration + (System.currentTimeMillis() - recogStartTime);
        recogStartTime = 0;
        handler.removeCallbacks(timeRunnable);
    }

    protected void sendecogStatus(int status) {
        handler.getAliRecordHelperListener().recogStatus(status);
    }

    private static void log(String content) {
        if (BuildConfig.DEBUG)
            LogUtils.e(TAG, content);
    }

    //======================================录音机实现类方法==============================================

    @Override
    public void audioData(byte[] bytes, long recordTime) {
        // TODO: 2024/12/16 读数据
        //使用录音数据和识别时间 作为音频波纹数据绘制内容
        handler.getAliRecordHelperListener().recogData(bytes);

        // 启动 WebSocket 连接并发送音频数据
        if (audioRecord != null) {
//            byte[] bytesa = new byte[1920];
//                int readSize = audioRecord.read(bytes, 0, 1920);
//                if (readSize > 0) {
//                    ByteString byteString = ByteString.of(bytes);
//                    webSocket.send(byteString);
//                    audioView.post(() -> audioView.setWaveData(bytes));
//                }

//            Log.d(TAG, "audioData: " + Arrays.toString(bytes));
            ByteString byteString = ByteString.of(bytes);
            webSocket.send(byteString);
        }
    }

    @Override
    public void recordStatus(int status) {
        log("录音状态：" + status);
    }

    @Override
    public void recordFinish(List<String> recordPcmList) {
        handler.getAliRecordHelperListener().recordFinish(recordPcmList);
    }

    @Override
    public void unExceptionInterrupt() {

    }


    // WebSocket 连接并发送音频数据
    private void sendAudioDataToWebSocket(byte[] bytes) {

    }

    // 发送第一步的JSON数据
    public String getMessage(boolean isSpeaking) {
        try {
            JSONObject obj = new JSONObject();
            obj.put("mode", MODE);
            JSONArray array = new JSONArray();
            String[] chunkList = CHUNK_SIZE.split(",");
            for (String s : chunkList) {
                array.put(Integer.valueOf(s.trim()));
            }
            obj.put("chunk_size", array);
            obj.put("chunk_interval", CHUNK_INTERVAL);
            obj.put("wav_name", "default");
            if (!hotWords.equals("")) {
                JSONObject hotwordsJSON = new JSONObject();
                // 分割每一行字符串
                String[] hotWordsList = hotWords.split("\n");
                for (String s : hotWordsList) {
                    if (s.equals("")) {
                        Log.w(TAG, "hotWords为空");
                        continue;
                    }
                    // 按照空格分割字符串
                    String[] hotWordsArray = s.split(" ");
                    if (hotWordsArray.length != 2) {
                        Log.w(TAG, "hotWords格式不正确");
                        continue;
                    }
                    hotwordsJSON.put(hotWordsArray[0], Integer.valueOf(hotWordsArray[1]));
                }
                obj.put("hotwords", hotwordsJSON.toString());
            }
            obj.put("wav_format", "pcm");
            obj.put("is_speaking", isSpeaking);
            return obj.toString();
        } catch (Exception e) {
            e.printStackTrace();
        }
        return "";
    }

}
